const FRAME_BYTES: usize = 16 * 1024 * 1024;

fn checksum_frame(seed: u8) -> u64 {
    let frame = [seed; FRAME_BYTES];
    frame.iter().map(|&b| b as u64).sum()
}

fn main() {
    println!("checksum {}", checksum_frame(1));
}
