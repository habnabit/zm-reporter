use chrono::prelude::*;
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
    pub recording: libc::timeval,
}

pub const Purpose_QUERY: Purpose = 0;
pub const Purpose_CAPTURE: Purpose = 1;
pub const Purpose_ANALYSIS: Purpose = 2;
pub type Purpose = ::std::os::raw::c_uint;
pub const Function_NONE: Function = 1;
pub const Function_MONITOR: Function = 2;
pub const Function_MODECT: Function = 3;
pub const Function_RECORD: Function = 4;
pub const Function_MOCORD: Function = 5;
pub const Function_NODECT: Function = 6;
pub type Function = ::std::os::raw::c_uint;
pub const CameraType_LOCAL: CameraType = 0;
pub const CameraType_REMOTE: CameraType = 1;
pub const CameraType_FILE: CameraType = 2;
pub const CameraType_FFMPEG: CameraType = 3;
pub const CameraType_LIBVLC: CameraType = 4;
pub const CameraType_CURL: CameraType = 5;
pub const CameraType_VNC: CameraType = 6;
pub type CameraType = ::std::os::raw::c_uint;
pub const Orientation_ROTATE_0: Orientation = 1;
pub const Orientation_ROTATE_90: Orientation = 2;
pub const Orientation_ROTATE_180: Orientation = 3;
pub const Orientation_ROTATE_270: Orientation = 4;
pub const Orientation_FLIP_HORI: Orientation = 5;
pub const Orientation_FLIP_VERT: Orientation = 6;
pub type Orientation = ::std::os::raw::c_uint;
pub const State_UNKNOWN: State = -1;
pub const State_IDLE: State = 0;
pub const State_PREALARM: State = 1;
pub const State_ALARM: State = 2;
pub const State_ALERT: State = 3;
pub const State_TAPE: State = 4;
pub type State = ::std::os::raw::c_int;
pub const VideoWriter_DISABLED: VideoWriter = 0;
pub const VideoWriter_X264ENCODE: VideoWriter = 1;
pub const VideoWriter_H264PASSTHROUGH: VideoWriter = 2;
pub type VideoWriter = ::std::os::raw::c_uint;
pub const Action_GET_SETTINGS: Action = 1;
pub const Action_SET_SETTINGS: Action = 2;
pub const Action_RELOAD: Action = 4;
pub const Action_SUSPEND: Action = 16;
pub const Action_RESUME: Action = 32;
pub type Action = ::std::os::raw::c_uint;
pub const EventCloseMode_CLOSE_TIME: EventCloseMode = 0;
pub const EventCloseMode_CLOSE_IDLE: EventCloseMode = 1;
pub const EventCloseMode_CLOSE_ALARM: EventCloseMode = 2;
pub type EventCloseMode = ::std::os::raw::c_uint;
pub const TriggerState_TRIGGER_CANCEL: TriggerState = 0;
pub const TriggerState_TRIGGER_ON: TriggerState = 1;
pub const TriggerState_TRIGGER_OFF: TriggerState = 2;
pub type TriggerState = ::std::os::raw::c_uint;

pub struct Monitor {
    shmem: *const u8,
    shared_data_ptr: *const SharedData,
    trigger_data_ptr: *const TriggerData,
    video_store_data_ptr: *const VideoStoreData,
    image_size: usize,
    frame_count: usize,
    timeval_ptr: *const libc::timeval,
    frame_ptr: *const u8,
}

impl Monitor {
    pub fn from_mmap_and_size(shmem: *const u8, size: isize) -> Self {
        let shmem_end = shmem.wrapping_offset(size);
        let shared_data_ptr = shmem as *const SharedData;
        let image_size = unsafe {
            (*shared_data_ptr).imagesize.load(Ordering::SeqCst)
        } as usize;
        let trigger_data_ptr = shared_data_ptr.wrapping_offset(1) as *const TriggerData;
        let video_store_data_ptr = trigger_data_ptr.wrapping_offset(1) as *const VideoStoreData;
        let rest = video_store_data_ptr.wrapping_offset(1) as *const u8;
        let rem = unsafe { shmem_end.offset_from(rest) };
        let each_left = image_size + std::mem::size_of::<libc::timeval>();
        let frame_count = rem as usize / each_left;
        let timeval_ptr = rest as *const libc::timeval;
        let mut frame_ptr = timeval_ptr.wrapping_offset(frame_count as isize) as *const u8;
        match frame_ptr.align_offset(64) {
            0 => {},
            n if n < 64 => frame_ptr = frame_ptr.wrapping_offset(n as isize),
            n => panic!("weird align_offset? {}", n),
        }
        Monitor {
            shmem, shared_data_ptr, trigger_data_ptr,
            video_store_data_ptr, image_size, frame_count, timeval_ptr, frame_ptr,
        }
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
    pub fn timevals(&self) -> &[libc::timeval] {
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
        let recorded_at = Utc.timestamp(when.tv_sec as i64, when.tv_usec as u32 * 1000);
        Frame { recorded_at, data }
    }
}

pub struct Frame {
    pub recorded_at: DateTime<Utc>,
    pub data: Vec<u8>,
}
