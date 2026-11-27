use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const CLAIM_LATENCY: Duration = Duration::from_millis(10);

fn claim(lock: &Path, node: &str) -> bool {
    match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(lock)
    {
        Ok(mut file) => file.write_all(node.as_bytes()).is_ok(),
        Err(_) => false,
    }
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
    println!(
        "nodes that believe they are leader: {}",
        leaders.load(Ordering::SeqCst)
    );
}
