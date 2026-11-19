async fn checkout() {
    let task = tokio::spawn(async { Err::<(), _>("disk full") });
    match task.await.expect("worker panicked") {
        Ok(()) => println!("saved"),
        Err(error) => println!("save failed: {error}"),
    }
}
fn main() {
    tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(checkout());
}
