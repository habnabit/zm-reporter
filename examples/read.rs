use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::{thread, time};
use zoneminder_shared_sys::{Frame, Monitor, MonitorSpec, MonitorStatus};

struct NewFrame {
    frame: Frame,
    state: u32,
}

enum Message {
    NewFrame(NewFrame),
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

fn do_one_frame(m: &Monitor) -> Result<Option<NewFrame>, Box<dyn std::error::Error>> {
    let state = m.shared_data().state.load(Ordering::SeqCst);
    if state == 0 || state == 4 {
        return Ok(None);
    }
    let last_write = m.shared_data().last_write_index.load(Ordering::SeqCst) as usize;
    let frame = m.frame(last_write);
    Ok(Some(NewFrame { frame, state }))
}

fn watch_monitor(tx: mpsc::SyncSender<Message>, spec: MonitorSpec) -> Result<(), Box<dyn std::error::Error>> {
    let _guard = SendOnDrop { id: spec.id, tx: tx.clone() };
    let m = Monitor::from_spec(spec)?;
    loop {
        if !m.valid() {
            return Ok(());
        }
        let delay = match do_one_frame(&m)? {
            Some(nf) => {
                tx.send(Message::NewFrame(nf))?;
                5_000
            },
            None => 100,
        };
        thread::sleep(time::Duration::from_millis(delay));
    }
}

struct Putter {
    client: reqwest::blocking::Client,
    url: url::Url,
}

impl Putter {
    fn put_img(&self, monitor_status: &MonitorStatus, NewFrame { frame, state }: NewFrame) -> Result<(), Box<dyn std::error::Error>> {
        let mut c = frame.start_jpeg();
        c.set_scan_optimization_mode(mozjpeg::ScanMode::AllComponentsTogether);
        c.set_mem_dest();
        c.start_compress();
        assert!(c.write_scanlines(&frame.data[..]));
        c.finish_compress();
        let jpeg = c.data_to_vec().unwrap();
        let r = self.client.put(self.url.clone())
            .body(jpeg)
            .query(&[
                ("id", frame.spec.id),
                ("state", state),
            ])
            .query(&[
                ("name", &monitor_status.name),
            ])
            .query(&[
                ("recorded_at", format!("{}", frame.recorded_at.format("%+"))),
            ])
            .send();
        println!("ran a PUT: {:#?}", r);
        if let Ok(resp) = r {
            println!("body: {:?}", resp.text());
        }
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let putter = {
        let url = std::env::args()
            .skip(1)
            .map(|s| s.parse::<url::Url>())
            .next()
            .unwrap()?;
        let client = reqwest::blocking::Client::new();
        Putter { client, url }
    };
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
            Ok(Message::NewFrame(nf)) => {
                println!("new frame {:?}", nf.frame.spec);
                let (_, monitor_status) = &specs[&nf.frame.spec.id];
                putter.put_img(monitor_status, nf)?;
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
