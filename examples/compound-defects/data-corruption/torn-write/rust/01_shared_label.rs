use std::sync::mpsc;
use std::thread;
const LABEL_BYTES: usize = 4;
const FIRST: &[u8] = b"ABCDEF";
const SECOND: &[u8] = b"123456";
const SECOND_OFFSET: usize = 2;
#[repr(C)]
struct Parcel { label: [u8; LABEL_BYTES], status: [u8; LABEL_BYTES] }
fn main() {
    let mut parcel = Box::new(Parcel { label: [0; LABEL_BYTES], status: *b"OPEN" });
    let address = parcel.label.as_mut_ptr() as usize;
    let (started, ready) = mpsc::channel();
    let (resume, proceed) = mpsc::channel();
    let first = thread::spawn(move || {
        let destination = address as *mut u8;
        for (index, byte) in FIRST.iter().enumerate() {
            unsafe { destination.add(index).write(*byte); }
            if index + 1 == LABEL_BYTES {
                started.send(()).unwrap();
                proceed.recv().unwrap();
            }
        }
    });
    ready.recv().unwrap();
    let second = thread::spawn(move || {
        let destination = address as *mut u8;
        unsafe { std::ptr::copy_nonoverlapping(SECOND.as_ptr(), destination.add(SECOND_OFFSET), SECOND.len()); }
    });
    second.join().unwrap();
    resume.send(()).unwrap();
    first.join().unwrap();
    println!("label {} status {}", String::from_utf8_lossy(&parcel.label), String::from_utf8_lossy(&parcel.status));
}
