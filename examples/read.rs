use std::os::unix::fs::MetadataExt;
use std::os::unix::io::AsRawFd;
use std::sync::atomic::Ordering;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let argv: Vec<_> = std::env::args().collect();
    let f = std::fs::File::open(&argv[1])?;
    let meta = f.metadata()?;
    println!("size {}", meta.size());
    let shmem = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            meta.size() as usize,
            libc::PROT_READ,
            libc::MAP_SHARED,
            f.as_raw_fd(),
            0,
        )
    };

    let m = zoneminder_shared_sys::Monitor::from_mmap_and_size(shmem as *const u8, meta.size() as isize);
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
