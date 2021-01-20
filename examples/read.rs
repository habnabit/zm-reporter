use std::sync::atomic::Ordering;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let argv: Vec<_> = std::env::args().collect();
    let f = std::fs::File::open(&argv[1])?;
    let m = zoneminder_shared_sys::Monitor::from_file(f)?;
    let last_write = m.shared_data().last_write_index.load(Ordering::SeqCst) as usize;
    let frame = m.frame(last_write);
    let fname = format!("{}.jpg", frame.recorded_at.format("%S"));
    println!("last frame {} {:?} {:?}", last_write, frame.recorded_at, fname);
    let mut c = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_EXT_RGBA);
    c.set_scan_optimization_mode(mozjpeg::ScanMode::AllComponentsTogether);
    c.set_size(1920, 1080);
    c.set_mem_dest();
    c.start_compress();
    assert!(c.write_scanlines(&frame.data[..]));
    c.finish_compress();
    let jpeg = c.data_to_vec().unwrap();
    std::io::copy(
        &mut &jpeg[..],
        &mut std::fs::File::create(fname)?,
    )?;
    //println!("{:#?}", m.shared_data());
    //println!("{:#?}", m.trigger_data());
    //println!("{:#?}", m.video_store_data());
    //println!("timevals {:#?}", m.timevals());
    Ok(())
}
