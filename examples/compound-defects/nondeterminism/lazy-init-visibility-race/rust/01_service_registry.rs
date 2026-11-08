use std::sync::{atomic::{AtomicBool, AtomicUsize, Ordering}, mpsc, Mutex};
use std::thread;
static PUBLISHED: AtomicBool = AtomicBool::new(false);
static INIT: Mutex<()> = Mutex::new(());
static READY: AtomicUsize = AtomicUsize::new(0);
const READY_VALUE: usize = 1;
fn main() {
    let (published, visible) = mpsc::channel();
    let (resume, proceed) = mpsc::channel();
    let worker = thread::spawn(move || {
        if !PUBLISHED.load(Ordering::Relaxed) {
            let _initializing = INIT.lock().unwrap();
            if !PUBLISHED.load(Ordering::Relaxed) {
                PUBLISHED.store(true, Ordering::Relaxed);
                published.send(()).unwrap();
                proceed.recv().unwrap();
                READY.store(READY_VALUE, Ordering::Relaxed);
            }
        }
    });
    visible.recv().unwrap();
    if PUBLISHED.load(Ordering::Relaxed) {
        println!("ready {}", READY.load(Ordering::Relaxed));
    }
    resume.send(()).unwrap();
    worker.join().unwrap();
}
