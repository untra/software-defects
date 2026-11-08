const PRICES: u64 = 64;

fn main() {
    let mut prices: Vec<u64> = Vec::with_capacity(1);
    prices.push(100);
    let first_index = 0;
    for p in 1..PRICES {
        prices.push(p * 1000);
    }
    let reused: Vec<u64> = vec![999; 1];
    let stale = prices[first_index];
    println!(
        "first price via saved pointer: {} (vector says {}, scratch {})",
        stale, prices[0], reused[0]
    );
}
