use std::sync::atomic::Ordering;
use std::{time, thread};

fn do_one(m: &zoneminder_shared_sys::Monitor) -> Result<bool, Box<dyn std::error::Error>> {
    let state = m.shared_data().state.load(Ordering::SeqCst);
    if state == 0 || state == 4 {
        return Ok(false)
    }
    let last_write = m.shared_data().last_write_index.load(Ordering::SeqCst) as usize;
    let frame = m.frame(last_write);
    let fname_txt = format!("{}.txt", frame.recorded_at.format("%S"));
    {
        use std::io::Write;
        let file = std::fs::File::create(fname_txt)?;
        let mut buf = std::io::BufWriter::new(file);
        write!(buf, "shared\n{:#?}\n\n", m.shared_data())?;
        write!(buf, "trigger\n{:#?}\n\n", m.trigger_data())?;
        write!(buf, "video_store\n{:#?}\n\n", m.video_store_data())?;
        write!(buf, "timevals\n{:#?}\n\n", m.timevals())?;
    }
    let fname_jpg = format!("{}.jpg", frame.recorded_at.format("%S"));
    println!("last frame {} {:?} {:?}", last_write, frame.recorded_at, fname_jpg);
    let mut c = frame.start_jpeg();
    c.set_scan_optimization_mode(mozjpeg::ScanMode::AllComponentsTogether);
    c.set_mem_dest();
    c.start_compress();
    assert!(c.write_scanlines(&frame.data[..]));
    c.finish_compress();
    let jpeg = c.data_to_vec().unwrap();
    std::io::copy(
        &mut &jpeg[..],
        &mut std::fs::File::create(fname_jpg)?,
    )?;
    Ok(true)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let argv: Vec<_> = std::env::args().collect();
    let which: u32 = argv[1].parse()?;
    zoneminder_shared_sys::load_env()?;
    let mut conn = zoneminder_shared_sys::mysql_connect()?;
    let specs = zoneminder_shared_sys::monitor_specs_from_mysql(&mut conn)?;
    let (spec, status) = &specs[&which];
    let m = zoneminder_shared_sys::Monitor::from_spec(*spec)?;
    let mut count = 0u64;
    loop {
        if !m.valid() {
            return Ok(())
        }
        if do_one(&m)? {
            thread::sleep(time::Duration::from_millis(5_000));
        } else {
            thread::sleep(time::Duration::from_millis(100));
        }
        count += 1;
        if count == 3125 {
            println!("{:?} 3125 ticks", chrono::Utc::now());
            count = 0;
        }
    }
}
