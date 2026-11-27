use std::sync::{mpsc, Arc, Mutex};
use std::thread;
const INITIAL: &str = "packing";
const UPDATED: &str = "shipped";
fn main() {
    let primary = Arc::new(Mutex::new(INITIAL));
    let replica = Arc::new(Mutex::new(INITIAL));
    *primary.lock().unwrap() = UPDATED;
    let (release, ready) = mpsc::channel();
    let copy = replica.clone();
    let pending = thread::spawn(move || {
        ready.recv().unwrap();
        *copy.lock().unwrap() = *primary.lock().unwrap();
    });
    println!("status {}", replica.lock().unwrap());
    release.send(()).unwrap();
    pending.join().unwrap();
}
