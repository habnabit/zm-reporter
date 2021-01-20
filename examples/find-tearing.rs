use std::sync::atomic::Ordering;

/* Updated in order:
 *  - image data
 *  - timestamp on image data
 *  - timestamp field
 *  - last write index
 *  - last write time
 */

macro_rules! abort_if {
    ($e1:expr, $e2:expr, $f:expr) => {
        let e1 = $e1;
        let e2 = $e2;
        let cond = ($f)(&e1, &e2);
        println!(concat!(
            stringify!($e1),
            " {:?} ",
            stringify!($e2),
            " {:?} {:?}",
        ), e1, e2, cond);
        if cond {
            return Ok(())
        }
    }
}

fn write_jpeg_1080p(data: &[u8], fname: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut c = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_EXT_RGBA);
    c.set_scan_optimization_mode(mozjpeg::ScanMode::AllComponentsTogether);
    c.set_size(1920, 1080);
    c.set_mem_dest();
    c.start_compress();
    assert!(c.write_scanlines(data));
    c.finish_compress();
    let jpeg = c.data_to_vec().unwrap();
    std::io::copy(
        &mut &jpeg[..],
        &mut std::fs::File::create(fname)?,
    )?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let argv: Vec<_> = std::env::args().collect();
    let f = std::fs::File::open(&argv[1])?;
    let m = zoneminder_shared_sys::Monitor::from_file(f)?;
    let last_write = m.shared_data().last_write_index.load(Ordering::SeqCst) as usize;
    let which = (last_write + 1) % m.frame_count();
    let (mut frame1, frame2) = {
        let last_write_time1 = m.shared_data().last_write_time.load(Ordering::SeqCst);
        let frame1 = m.frame(which);
        {
            use std::{time, thread};
            thread::sleep(time::Duration::from_millis(20));
        }
        let frame2 = m.frame(which);
        let last_write_time2 = m.shared_data().last_write_time.load(Ordering::SeqCst);
        let last_write2 = m.shared_data().last_write_index.load(Ordering::SeqCst) as usize;
        abort_if!(frame1.recorded_at, frame2.recorded_at, |a, b| a == b);
        abort_if!(last_write_time1, last_write_time2, |a, b| a == b);
        abort_if!(last_write, last_write2, |a, b| a == b);
        if &frame1.data == &frame2.data {
            println!("data equal too");
            return Ok(());
        }
        (frame1, frame2)
    };
    let fname_out = format!("{}.jpg", frame1.recorded_at.format("%S"));
    let fname_mask = format!("{}_mask.jpg", frame1.recorded_at.format("%S"));
    println!("last frame {} ({}) {:?} {:?}", last_write, which, frame1.recorded_at, fname_out);
    write_jpeg_1080p(&frame2.data[..], &fname_out)?;
    for (e, (dest, src)) in frame1.data.iter_mut().zip(&frame2.data).enumerate() {
        if e % 4 == 3 {
            *dest = 255;
        } else {
            *dest ^= *src;
        }
    }
    write_jpeg_1080p(&frame1.data[..], &fname_mask)?;
    //println!("{:#?}", m.shared_data());
    //println!("{:#?}", m.trigger_data());
    //println!("{:#?}", m.video_store_data());
    //println!("timevals {:#?}", m.timevals());
    Ok(())
}
