use std::sync::{Arc, Barrier, Mutex};
use std::thread;
const CLERKS: usize = 2;
fn main() {
    let parcels = Arc::new(Mutex::new(0));
    let loaded = Arc::new(Barrier::new(CLERKS));
    let mut clerks = Vec::new();
    for _ in 0..CLERKS {
        let (parcels, loaded) = (parcels.clone(), loaded.clone());
        clerks.push(thread::spawn(move || {
            let mut count = *parcels.lock().unwrap();
            loaded.wait();
            count += 1;
            *parcels.lock().unwrap() = count;
        }));
    }
    for clerk in clerks { clerk.join().unwrap(); }
    println!("parcels {}", parcels.lock().unwrap());
}
