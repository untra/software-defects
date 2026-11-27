use std::sync::{Arc, Barrier, Mutex};
use std::thread;

const BUYERS: usize = 5;
const STOCK: u32 = 1;

fn reserve(stock: &Mutex<u32>, checked: &Barrier) -> bool {
    checked.wait();
    let mut available = stock.lock().unwrap();
    if *available > 0 {
        *available -= 1;
        return true;
    }
    false
}

fn main() {
    let stock = Arc::new(Mutex::new(STOCK));
    let checked = Arc::new(Barrier::new(BUYERS));
    let buyers: Vec<_> = (0..BUYERS)
        .map(|_| {
            let stock = Arc::clone(&stock);
            let checked = Arc::clone(&checked);
            thread::spawn(move || reserve(&stock, &checked))
        })
        .collect();
    let sold = buyers
        .into_iter()
        .map(|b| b.join().unwrap())
        .filter(|&ok| ok)
        .count();
    println!("sold {} units with {} in stock", sold, STOCK);
}
