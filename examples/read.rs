use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::{thread, time};
use zoneminder_shared_sys::{Frame, Monitor, MonitorSpec};

enum Message {
    NewFrame(Frame),
    LoopDone(u32),
}

struct SendOnDrop {
    id: u32,
    tx: mpsc::SyncSender<Message>,
}

impl Drop for SendOnDrop {
    fn drop(&mut self) {
        let _ = self.tx.send(Message::LoopDone(self.id));
    }
}

fn do_one_frame(m: &Monitor) -> Result<Option<Frame>, Box<dyn std::error::Error>> {
    let state = m.shared_data().state.load(Ordering::SeqCst);
    if state == 0 || state == 4 {
        return Ok(None);
    }
    let last_write = m.shared_data().last_write_index.load(Ordering::SeqCst) as usize;
    let frame = m.frame(last_write);
    Ok(Some(frame))
}

fn watch_monitor(tx: mpsc::SyncSender<Message>, spec: MonitorSpec) -> Result<(), Box<dyn std::error::Error>> {
    let _guard = SendOnDrop { id: spec.id, tx: tx.clone() };
    let m = Monitor::from_spec(spec)?;
    loop {
        if !m.valid() {
            return Ok(());
        }
        let delay = match do_one_frame(&m)? {
            Some(f) => {
                tx.send(Message::NewFrame(f))?;
                5_000
            },
            None => 100,
        };
        thread::sleep(time::Duration::from_millis(delay));
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    zoneminder_shared_sys::load_env()?;
    let mut conn = zoneminder_shared_sys::mysql_connect()?;
    let mut specs = zoneminder_shared_sys::monitor_specs_from_mysql(&mut conn)?;
    let mut last_fetch = time::Instant::now();
    let mut running = BTreeMap::<u32, thread::JoinHandle<()>>::new();
    let one_minute = time::Duration::from_secs(60);
    let (tx, rx) = mpsc::sync_channel(0);
    loop {
        let mut refetch = false;
        let to_start = specs
            .keys()
            .collect::<BTreeSet<_>>()
            .difference(&running.keys().collect())
            .map(|&&i| i)
            .collect::<BTreeSet<_>>();
        for i in to_start {
            println!("starting {}", i);
            let tx = tx.clone();
            let &(spec, _) = &specs[&i];
            running.insert(i, thread::spawn(move || watch_monitor(tx, spec).expect("watch failed")));
        }
        match rx.recv_timeout(one_minute) {
            Ok(Message::LoopDone(i)) => running.remove(&i).map(|h| {
                let r = h.join();
                println!("joined {}: {:?}", i, r);
                refetch = true;
            }).unwrap_or(()),
            Ok(Message::NewFrame(frame)) => {
                println!("new frame {:?}", frame.spec);
            },
            Err(mpsc::RecvTimeoutError::Timeout) => (),
            Err(mpsc::RecvTimeoutError::Disconnected) => unreachable!(),
        }
        let now = time::Instant::now();
        if refetch || now.saturating_duration_since(last_fetch) > one_minute {
            specs = zoneminder_shared_sys::monitor_specs_from_mysql(&mut conn)?;
            last_fetch = now;
        }
    }
}
