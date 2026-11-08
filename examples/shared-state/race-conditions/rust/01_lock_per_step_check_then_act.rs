use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const BUYERS: usize = 5;
const STOCK: u32 = 1;
const RESERVE_LATENCY: Duration = Duration::from_millis(10);

fn reserve(stock: &Mutex<u32>) -> bool {
    let available = *stock.lock().unwrap();
    if available > 0 {
        thread::sleep(RESERVE_LATENCY);
        *stock.lock().unwrap() = available - 1;
        return true;
    }
    false
}

fn main() {
    let stock = Arc::new(Mutex::new(STOCK));
    let buyers: Vec<_> = (0..BUYERS)
        .map(|_| {
            let stock = Arc::clone(&stock);
            thread::spawn(move || reserve(&stock))
        })
        .collect();
    let sold = buyers.into_iter().map(|b| b.join().unwrap()).filter(|&ok| ok).count();
    println!("sold {} units with {} in stock", sold, STOCK);
}
