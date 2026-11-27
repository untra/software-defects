use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

const WATCHDOG: Duration = Duration::from_millis(500);

struct Inventory {
    stock: Mutex<u32>,
}

impl Inventory {
    fn available(&self) -> u32 {
        *self.stock.lock().unwrap()
    }

    fn take(&self, n: u32) -> bool {
        let mut stock = self.stock.lock().unwrap();
        if self.available() < n {
            return false;
        }
        *stock -= n;
        true
    }
}

fn main() {
    let inventory = Arc::new(Inventory { stock: Mutex::new(10) });
    let (done, finished) = mpsc::channel();
    let worker = Arc::clone(&inventory);
    thread::spawn(move || done.send(worker.take(3)).unwrap());
    match finished.recv_timeout(WATCHDOG) {
        Ok(taken) => println!("taken: {}", taken),
        Err(_) => println!("take() still blocked after {:?}", WATCHDOG),
    }
}
