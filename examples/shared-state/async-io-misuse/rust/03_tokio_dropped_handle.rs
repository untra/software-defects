async fn checkout() {
    let task = tokio::spawn(async { Err::<(), _>("disk full") });
    drop(task);
    println!("saved");
}
fn main() {
    tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(checkout());
}
