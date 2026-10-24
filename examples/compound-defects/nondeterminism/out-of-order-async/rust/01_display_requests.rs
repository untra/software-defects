use std::sync::{mpsc, Arc, Mutex};
use std::thread;
fn main() {
    let shown = Arc::new(Mutex::new(""));
    let (release, ready) = mpsc::channel();
    let earlier_shown = shown.clone();
    let earlier = thread::spawn(move || {
        ready.recv().unwrap();
        *earlier_shown.lock().unwrap() = "orchard";
    });
    let later_shown = shown.clone();
    thread::spawn(move || {
        *later_shown.lock().unwrap() = "harbor";
    }).join().unwrap();
    release.send(()).unwrap();
    earlier.join().unwrap();
    println!("shown {}", shown.lock().unwrap());
}
