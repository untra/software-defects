use std::time::Instant;

const PAYLOAD_BYTES: u64 = 4096;

fn main() {
    let started = Instant::now();
    let checksum: u64 = (0..PAYLOAD_BYTES).sum();
    let elapsed_secs = started.elapsed().as_secs();
    let throughput = PAYLOAD_BYTES / elapsed_secs;
    println!("checksum {} throughput {} B/s", checksum, throughput);
}
