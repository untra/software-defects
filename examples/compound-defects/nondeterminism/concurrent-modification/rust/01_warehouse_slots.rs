use std::sync::{mpsc, Arc, Mutex};
use std::thread;
const BEFORE: [u32; 3] = [1, 2, 3];
const AFTER: [u32; 3] = [10, 20, 30];
fn main() {
    let slots = Arc::new(Mutex::new(BEFORE));
    let (scanned, ready) = mpsc::channel();
    let (resume, proceed) = mpsc::channel();
    let reader_slots = slots.clone();
    let reader = thread::spawn(move || {
        let mut total = 0;
        for index in 0..BEFORE.len() {
            total += reader_slots.lock().unwrap()[index];
            if index == 0 {
                scanned.send(()).unwrap();
                proceed.recv().unwrap();
            }
        }
        total
    });
    ready.recv().unwrap();
    let writer = thread::spawn(move || { *slots.lock().unwrap() = AFTER; });
    writer.join().unwrap();
    resume.send(()).unwrap();
    println!("units {}", reader.join().unwrap());
}
