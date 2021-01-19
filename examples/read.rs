use std::os::unix::fs::MetadataExt;
use std::os::unix::io::AsRawFd;

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
    let shmem_end = shmem.wrapping_offset(meta.size() as isize);
    println!("mapped {:?} {:?}", shmem, shmem_end);
    let shared_data_ptr = shmem as *const zoneminder_shared_sys::SharedData;
    let shared_data = unsafe { std::ptr::read_volatile(shared_data_ptr) };
    println!("{:#?}", shared_data);
    let trigger_data_ptr = shared_data_ptr.wrapping_offset(1) as *const zoneminder_shared_sys::TriggerData;
    let trigger_data = unsafe { std::ptr::read_volatile(trigger_data_ptr) };
    println!("{:#?}", trigger_data);
    let video_store_data_ptr = trigger_data_ptr.wrapping_offset(1) as *const zoneminder_shared_sys::VideoStoreData;
    let video_store_data = unsafe { std::ptr::read_volatile(video_store_data_ptr) };
    println!("{:#?}", video_store_data);
    let rest = video_store_data_ptr.wrapping_offset(1) as *const libc::c_void;
    let rem = unsafe { shmem_end.offset_from(rest) };
    let each_left = shared_data.imagesize.load(std::sync::atomic::Ordering::SeqCst) as usize + std::mem::size_of::<libc::timeval>();
    let frame_count = rem as usize / each_left;
    println!("{:?} {:?}", rest, (video_store_data_ptr as *const u8).wrapping_offset(std::mem::size_of::<zoneminder_shared_sys::VideoStoreData>() as isize));
    println!("remainder {} each_left {} / {} % {}", rem, each_left, frame_count, rem as usize % each_left);
    let mut timevals = vec![libc::timeval { tv_sec: 0, tv_usec: 0 }; frame_count];
    unsafe {
        std::ptr::copy_nonoverlapping(rest as *const libc::timeval, timevals.as_mut_ptr(), frame_count);
    }
    println!("timevals {:?}", timevals);
    Ok(())
}
