use std::sync::mpsc;
use std::thread;
const CHUNK_BYTES: usize = 4;
const SHIPMENT: [&str; 2] = ["pears", "figs-large"];
fn main() {
    let (send, receive) = mpsc::channel();
    let producer = thread::spawn(move || {
        let bytes = SHIPMENT.join("").into_bytes();
        for chunk in bytes.chunks(CHUNK_BYTES) { send.send(chunk.to_vec()).unwrap(); }
    });
    let mut records = Vec::new();
    for chunk in receive { records.push(String::from_utf8(chunk).unwrap()); }
    producer.join().unwrap();
    println!("records {:?}", records);
}
