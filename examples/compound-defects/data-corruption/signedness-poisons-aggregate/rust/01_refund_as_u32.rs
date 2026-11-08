use std::sync::Mutex;

static LEDGER: Mutex<u64> = Mutex::new(0);

const OPENING: u64 = 10;
const REFUND: i32 = -1;

fn main() {
    let credit = u64::from(REFUND as u32);
    let mut balance = LEDGER.lock().unwrap();
    *balance += OPENING;
    *balance += credit;
    println!("balance {balance}");
}
