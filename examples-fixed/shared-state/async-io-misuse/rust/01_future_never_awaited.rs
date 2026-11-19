async fn send_receipt(outbox: &mut Vec<String>) {
    tokio::task::yield_now().await;
    outbox.push("receipt".into());
}
async fn checkout() {
    let mut outbox = Vec::new();
    send_receipt(&mut outbox).await;
    println!("receipts sent: {}", outbox.len());
}
fn main() {
    tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(checkout());
}
