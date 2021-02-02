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
    let mut c = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_EXT_RGB);
    c.set_scan_optimization_mode(mozjpeg::ScanMode::AllComponentsTogether);
    c.set_size(1920, 1080);
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
    let f = std::fs::File::open(&argv[1])?;
    let m = zoneminder_shared_sys::Monitor::from_file(f)?;
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
