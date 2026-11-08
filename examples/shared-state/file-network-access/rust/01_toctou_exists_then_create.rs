use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const CLAIM_LATENCY: Duration = Duration::from_millis(10);

fn claim(lock: &Path, node: &str) -> bool {
    if !lock.exists() {
        thread::sleep(CLAIM_LATENCY);
        return fs::write(lock, node).is_ok();
    }
    false
}

fn main() {
    let lock = std::env::temp_dir().join(format!("leader-{}.lock", std::process::id()));
    let leaders = Arc::new(AtomicUsize::new(0));
    let nodes: Vec<_> = ["node-a", "node-b", "node-c"]
        .into_iter()
        .map(|node| {
            let (lock, leaders) = (lock.clone(), Arc::clone(&leaders));
            thread::spawn(move || {
                if claim(&lock, node) {
                    leaders.fetch_add(1, Ordering::SeqCst);
                }
            })
        })
        .collect();
    for n in nodes {
        n.join().unwrap();
    }
    let _ = fs::remove_file(&lock);
    println!("nodes that believe they are leader: {}", leaders.load(Ordering::SeqCst));
}
