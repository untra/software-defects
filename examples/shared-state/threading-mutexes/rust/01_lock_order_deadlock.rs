use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

const LOCK_GAP: Duration = Duration::from_millis(10);
const WATCHDOG: Duration = Duration::from_millis(500);

fn transfer(from: &Mutex<i64>, to: &Mutex<i64>, amount: i64) {
    let mut source = from.lock().unwrap();
    thread::sleep(LOCK_GAP);
    let mut target = to.lock().unwrap();
    *source -= amount;
    *target += amount;
}

fn main() {
    let alice = Arc::new(Mutex::new(100));
    let bob = Arc::new(Mutex::new(100));
    let (done, finished) = mpsc::channel();
    for (from, to, amount) in [(Arc::clone(&alice), Arc::clone(&bob), 10), (Arc::clone(&bob), Arc::clone(&alice), 20)] {
        let done = done.clone();
        thread::spawn(move || {
            transfer(&from, &to, amount);
            done.send(()).unwrap();
        });
    }
    for _ in 0..2 {
        if finished.recv_timeout(WATCHDOG).is_err() {
            println!("transfers still blocked after {:?}", WATCHDOG);
            std::process::exit(0);
        }
    }
    println!("balances {} {}", alice.lock().unwrap(), bob.lock().unwrap());
}
