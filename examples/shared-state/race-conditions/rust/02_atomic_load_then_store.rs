use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

const WORKERS: usize = 8;
const HITS_PER_WORKER: usize = 1_000_000;

fn record(hits: &AtomicUsize) {
    let current = hits.load(Ordering::SeqCst);
    hits.store(current + 1, Ordering::SeqCst);
}

fn main() {
    let hits = Arc::new(AtomicUsize::new(0));
    let workers: Vec<_> = (0..WORKERS)
        .map(|_| {
            let hits = Arc::clone(&hits);
            thread::spawn(move || {
                for _ in 0..HITS_PER_WORKER {
                    record(&hits);
                }
            })
        })
        .collect();
    for w in workers {
        w.join().unwrap();
    }
    println!("recorded {} of {} hits", hits.load(Ordering::SeqCst), WORKERS * HITS_PER_WORKER);
}
