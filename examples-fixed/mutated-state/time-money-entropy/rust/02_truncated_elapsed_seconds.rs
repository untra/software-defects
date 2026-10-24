use std::time::Instant;

const PAYLOAD_BYTES: u64 = 4096;

fn main() {
    let started = Instant::now();
    let checksum: u64 = (0..PAYLOAD_BYTES).sum();
    let elapsed_secs = started.elapsed().as_secs_f64();
    let throughput = if elapsed_secs > 0.0 {
        PAYLOAD_BYTES as f64 / elapsed_secs
    } else {
        0.0
    };
    println!("checksum {} throughput {} B/s", checksum, throughput);
}
