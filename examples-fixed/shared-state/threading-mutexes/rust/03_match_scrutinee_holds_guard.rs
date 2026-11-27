use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

const WATCHDOG: Duration = Duration::from_millis(500);

fn retry_next(queue: &Mutex<Vec<String>>) {
    let job = queue.lock().unwrap().pop();
    match job {
        Some(job) => queue.lock().unwrap().push(format!("{}-retry", job)),
        None => {}
    }
}

fn main() {
    let queue = Arc::new(Mutex::new(vec!["send-email".to_string()]));
    let (done, finished) = mpsc::channel();
    let worker = Arc::clone(&queue);
    thread::spawn(move || {
        retry_next(&worker);
        done.send(()).unwrap();
    });
    match finished.recv_timeout(WATCHDOG) {
        Ok(()) => println!("queue: {:?}", queue.lock().unwrap()),
        Err(_) => println!("retry_next still blocked after {:?}", WATCHDOG),
    }
}
