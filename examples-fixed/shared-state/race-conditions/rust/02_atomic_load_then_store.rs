use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;

const WORKERS: usize = 8;
const HITS_PER_WORKER: usize = 100;

fn record(hits: &AtomicUsize, checked: &Barrier) {
    checked.wait();
    hits.fetch_add(1, Ordering::SeqCst);
}

fn main() {
    let hits = Arc::new(AtomicUsize::new(0));
    let checked = Arc::new(Barrier::new(WORKERS));
    let workers: Vec<_> = (0..WORKERS)
        .map(|_| {
            let hits = Arc::clone(&hits);
            let checked = Arc::clone(&checked);
            thread::spawn(move || {
                for _ in 0..HITS_PER_WORKER {
                    record(&hits, &checked);
                    checked.wait();
                }
            })
        })
        .collect();
    for w in workers {
        w.join().unwrap();
    }
    println!(
        "recorded {} of {} hits",
        hits.load(Ordering::SeqCst),
        WORKERS * HITS_PER_WORKER
    );
}
