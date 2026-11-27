use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const WORKERS: u32 = 4;
const FETCH_LATENCY: Duration = Duration::from_millis(50);

fn refresh(cache: &Mutex<Vec<String>>, key: u32) {
    thread::sleep(FETCH_LATENCY);
    let mut entries = cache.lock().unwrap();
    entries.push(format!("fresh-{}", key));
}

fn main() {
    let cache = Arc::new(Mutex::new(Vec::new()));
    let start = Instant::now();
    let workers: Vec<_> = (0..WORKERS)
        .map(|k| {
            let cache = Arc::clone(&cache);
            thread::spawn(move || refresh(&cache, k))
        })
        .collect();
    for w in workers {
        w.join().unwrap();
    }
    let serialized = start.elapsed() >= FETCH_LATENCY * WORKERS;
    println!(
        "{} parallel refreshes ran one at a time: {}",
        WORKERS, serialized
    );
}
