use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const TASKS: usize = 500;
const TASK_DURATION: Duration = Duration::from_millis(50);
const CPU_THREADS: usize = 8;

fn process_uploads(live: Arc<AtomicUsize>, peak: Arc<AtomicUsize>) {
    let handles: Vec<_> = (0..TASKS)
        .map(|_| {
            let (live, peak) = (Arc::clone(&live), Arc::clone(&peak));
            thread::spawn(move || {
                let now = live.fetch_add(1, Ordering::SeqCst) + 1;
                peak.fetch_max(now, Ordering::SeqCst);
                thread::sleep(TASK_DURATION);
                live.fetch_sub(1, Ordering::SeqCst);
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
}

fn main() {
    let peak = Arc::new(AtomicUsize::new(0));
    process_uploads(Arc::new(AtomicUsize::new(0)), Arc::clone(&peak));
    println!("peak OS threads exceeded {}: {}", CPU_THREADS, peak.load(Ordering::SeqCst) > CPU_THREADS);
}
