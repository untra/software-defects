fn main() {
    let seconds: u64 = 5_000_000_000;
    let truncated = seconds as u32;
    println!("timeout seconds: {truncated}");
}
