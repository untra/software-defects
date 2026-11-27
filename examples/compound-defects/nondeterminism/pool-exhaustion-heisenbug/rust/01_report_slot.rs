use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Instant;
const CAPACITY: usize = 1;
fn main() {
    let occupied = Arc::new(Mutex::new(0));
    let (held, ready) = mpsc::channel();
    let (release, proceed) = mpsc::channel();
    let holder_pool = occupied.clone();
    let holder = thread::spawn(move || {
        *holder_pool.lock().unwrap() += 1;
        let started = Instant::now();
        held.send(()).unwrap();
        proceed.recv().unwrap();
        let _duration = started.elapsed();
        *holder_pool.lock().unwrap() -= 1;
    });
    ready.recv().unwrap();
    println!("pool busy {}", *occupied.lock().unwrap() == CAPACITY);
    release.send(()).unwrap();
    holder.join().unwrap();
}
