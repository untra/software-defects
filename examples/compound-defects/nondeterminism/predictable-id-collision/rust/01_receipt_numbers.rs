use std::collections::HashMap;
use std::sync::{Arc, Barrier, Mutex};
use std::thread;
const MILLIS: u64 = 1700000000000;
const CALLERS: usize = 2;
fn main() {
    let receipts = Arc::new(Mutex::new(HashMap::new()));
    let checked = Arc::new(Barrier::new(CALLERS));
    let mut workers = Vec::new();
    for customer in ["orchard", "harbor"] {
        let (receipts, checked) = (receipts.clone(), checked.clone());
        workers.push(thread::spawn(move || {
            let id = format!("receipt-{MILLIS}");
            let available = !receipts.lock().unwrap().contains_key(&id);
            checked.wait();
            if available { receipts.lock().unwrap().insert(id, customer); }
        }));
    }
    for worker in workers { worker.join().unwrap(); }
    println!("receipts {}", receipts.lock().unwrap().len());
}
