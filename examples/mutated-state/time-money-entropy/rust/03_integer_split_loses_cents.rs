const BILL_CENTS: u64 = 1000;
const DINERS: u64 = 3;

fn split(total_cents: u64, ways: u64) -> Vec<u64> {
    vec![total_cents / ways; ways as usize]
}

fn main() {
    let shares = split(BILL_CENTS, DINERS);
    let collected: u64 = shares.iter().sum();
    println!("shares {:?} collected {} of {}", shares, collected, BILL_CENTS);
}
