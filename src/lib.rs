use chrono::prelude::*;
use nix::sys::mman::{mmap, munmap, MapFlags, ProtFlags};
use nix::sys::time::TimeVal;
use std::fs::File;
use std::os::unix::fs::MetadataExt;
use std::os::unix::io::AsRawFd;
use std::sync::atomic::*;

#[repr(C)]
#[derive(Debug)]
pub struct SharedData {
    pub size: AtomicU32,
    pub last_write_index: AtomicU32,
    pub last_read_index: AtomicU32,
    pub state: AtomicU32,
    pub last_event: AtomicU64,
    pub action: AtomicU32,
    pub brightness: AtomicI32,
    pub hue: AtomicI32,
    pub colour: AtomicI32,
    pub contrast: AtomicI32,
    pub alarm_x: AtomicI32,
    pub alarm_y: AtomicI32,
    pub valid: AtomicU8,
    pub active: AtomicU8,
    pub signal: AtomicU8,
    pub format: AtomicU8,
    pub imagesize: AtomicU32,
    pub last_frame_score: AtomicU32,
    pub startup_time: AtomicU64,
    pub last_write_time: AtomicU64,
    pub last_read_time: AtomicU64,
    pub control_state: [u8; 256],
    pub alarm_cause: [u8; 256],
}

#[repr(C)]
#[derive(Debug)]
pub struct TriggerData {
    pub size: AtomicU32,
    pub trigger_state: AtomicU32,
    pub trigger_score: AtomicU32,
    pub padding: AtomicU32,
    pub trigger_cause: [u8; 32],
    pub trigger_text: [u8; 256],
    pub trigger_showtext: [u8; 256],
}

#[repr(C)]
#[derive(Debug)]
pub struct VideoStoreData {
    pub size: AtomicU32,
    pub current_event: AtomicU64,
    pub event_file: [u8; 4096],
    pub recording: TimeVal,
}

pub struct Monitor {
    _file: File,
    map_size: usize,
    shmem: *const u8,
    shared_data_ptr: *const SharedData,
    trigger_data_ptr: *const TriggerData,
    video_store_data_ptr: *const VideoStoreData,
    image_size: usize,
    frame_count: usize,
    timeval_ptr: *const TimeVal,
    frame_ptr: *const u8,
}

impl Drop for Monitor {
    fn drop(&mut self) {
        match unsafe { munmap(self.shmem as *mut std::ffi::c_void, self.map_size) } {
            Ok(()) => (),
            Err(e) => panic!("failed to munmap {:?}", e),
        }
    }
}

impl Monitor {
    pub fn from_file(file: File) -> Result<Self, Box<dyn std::error::Error>> {
        let meta = file.metadata()?;
        let map_size = meta.size() as usize;
        let shmem = unsafe {
            mmap(
                std::ptr::null_mut(),
                map_size,
                ProtFlags::PROT_READ,
                MapFlags::MAP_SHARED,
                file.as_raw_fd(),
                0,
            )
        }? as *const u8;
        let shmem_end = shmem.wrapping_offset(meta.size() as isize);
        debug_assert!(shmem_end > shmem);
        let shared_data_ptr = shmem as *const SharedData;
        let image_size = unsafe {
            (*shared_data_ptr).imagesize.load(Ordering::SeqCst)
        } as usize;
        let trigger_data_ptr = shared_data_ptr.wrapping_offset(1) as *const TriggerData;
        let video_store_data_ptr = trigger_data_ptr.wrapping_offset(1) as *const VideoStoreData;
        let rest = video_store_data_ptr.wrapping_offset(1) as *const u8;
        let rem = unsafe { shmem_end.offset_from(rest) };
        let each_left = image_size + std::mem::size_of::<TimeVal>();
        let frame_count = rem as usize / each_left;
        let timeval_ptr = rest as *const TimeVal;
        let mut frame_ptr = timeval_ptr.wrapping_offset(frame_count as isize) as *const u8;
        match frame_ptr.align_offset(64) {
            0 => {},
            n if n < 64 => frame_ptr = frame_ptr.wrapping_offset(n as isize),
            n => panic!("weird align_offset? {}", n),
        }
        Ok(Monitor {
            _file: file,
            map_size, shmem, shared_data_ptr, trigger_data_ptr,
            video_store_data_ptr, image_size, frame_count, timeval_ptr, frame_ptr,
        })
    }

    pub fn shared_data(&self) -> &SharedData {
        unsafe { &*self.shared_data_ptr }
    }
    pub fn trigger_data(&self) -> &TriggerData {
        unsafe { &*self.trigger_data_ptr }
    }
    pub fn video_store_data(&self) -> &VideoStoreData {
        unsafe { &*self.video_store_data_ptr }
    }
    pub fn timevals(&self) -> &[TimeVal] {
        unsafe { std::slice::from_raw_parts(self.timeval_ptr, self.frame_count as usize) }
    }
    pub fn frame(&self, n: usize) -> Frame {
        if n >= self.frame_count {
            panic!("frame {} exceeds {}", n, self.frame_count)
        }
        let frame = self.frame_ptr.wrapping_offset((self.image_size * n) as isize);
        let mut data = vec![0u8; self.image_size];
        let frame_slice = unsafe { std::slice::from_raw_parts(frame as *const u8, self.image_size) };
        data.copy_from_slice(frame_slice);
        let when = self.timevals()[n];
        let recorded_at = Utc.timestamp(when.tv_sec() as i64, when.tv_usec() as u32 * 1000);
        Frame { recorded_at, data }
    }
}

pub struct Frame {
    pub recorded_at: DateTime<Utc>,
    pub data: Vec<u8>,
}
