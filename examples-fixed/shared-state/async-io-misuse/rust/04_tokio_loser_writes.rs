use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
async fn checkout() {
    let writes = Arc::new(AtomicUsize::new(0));
    let (release, wait) = tokio::sync::oneshot::channel::<()>();
    let counter = Arc::clone(&writes);
    let loser = tokio::spawn(async move {
        let _ = wait.await;
        counter.fetch_add(1, Ordering::SeqCst);
    });
    tokio::task::yield_now().await;
    loser.abort();
    assert!(loser.await.unwrap_err().is_cancelled());
    let _ = release.send(());
    println!("late writes: {}", writes.load(Ordering::SeqCst));
}
fn main() {
    tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(checkout());
}
