const DEPOSIT: f64 = 0.10;
const DEPOSITS: u32 = 10;
const TARGET: f64 = 1.00;

fn main() {
    let mut balance = 0.0_f64;
    for _ in 0..DEPOSITS {
        balance += DEPOSIT;
    }
    if balance == TARGET {
        println!("goal reached");
    } else {
        println!("balance {:.17}, goal not reached", balance);
    }
}
