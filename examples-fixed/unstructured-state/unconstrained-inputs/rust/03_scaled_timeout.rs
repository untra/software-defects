fn main() {
    let seconds: u64 = 5_000_000_000;
    let Ok(truncated) = u32::try_from(seconds) else {
        println!("timeout rejected");
        return;
    };
    println!("timeout seconds: {truncated}");
}
