const BILL_CENTS: u64 = 1000;
const DINERS: u64 = 3;

fn split(total_cents: u64, ways: u64) -> Vec<u64> {
    if ways == 0 {
        return Vec::new();
    }
    (0..ways)
        .map(|i| total_cents / ways + u64::from(i < total_cents % ways))
        .collect()
}

fn main() {
    let shares = split(BILL_CENTS, DINERS);
    let collected: u64 = shares.iter().sum();
    println!(
        "shares {:?} collected {} of {}",
        shares, collected, BILL_CENTS
    );
}
