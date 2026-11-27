const DEPOSIT: u64 = 10;
const DEPOSITS: u32 = 10;
const TARGET: u64 = 100;

fn main() {
    let mut balance = 0_u64;
    for _ in 0..DEPOSITS {
        balance += DEPOSIT;
    }
    if balance == TARGET {
        println!("goal reached");
    } else {
        println!("balance {}, goal not reached", balance);
    }
}
