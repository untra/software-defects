async fn checkout() {
    let (mut writer, mut reader) = tokio::io::duplex(16);
    let task = tokio::spawn(async move {
        use tokio::io::AsyncWriteExt;
        writer.write_all(b"receipt").await
    });
    use tokio::io::AsyncReadExt;
    let mut receipt = [0; 7];
    reader.read_exact(&mut receipt).await.expect("receipt read");
    let result = task.await.expect("worker panicked");
    println!("receipt delivered: {}", result.is_ok());
}
fn main() {
    tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(checkout());
}
