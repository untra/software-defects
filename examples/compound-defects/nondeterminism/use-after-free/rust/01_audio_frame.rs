use std::ffi::c_void;
use std::io::{self, Write};
use std::ptr;

const FRAME_BYTES: usize = 16 * 1024;
const READ_WRITE: i32 = 3;
const PRIVATE: i32 = 2;
#[cfg(target_os = "macos")]
const ANONYMOUS: i32 = 0x1000;
#[cfg(not(target_os = "macos"))]
const ANONYMOUS: i32 = 0x20;

extern "C" {
    fn mmap(address: *mut c_void, length: usize, protection: i32, flags: i32, fd: i32, offset: i64) -> *mut c_void;
    fn munmap(address: *mut c_void, length: usize) -> i32;
}

fn main() {
    unsafe {
        let frame = mmap(ptr::null_mut(), FRAME_BYTES, READ_WRITE, PRIVATE | ANONYMOUS, -1, 0);
        assert_ne!(frame as isize, -1);
        let playback = frame.cast::<u8>();
        assert_eq!(munmap(frame, FRAME_BYTES), 0);
        println!("playback requested");
        io::stdout().flush().unwrap();
        println!("sample {}", playback.read_volatile());
    }
}
