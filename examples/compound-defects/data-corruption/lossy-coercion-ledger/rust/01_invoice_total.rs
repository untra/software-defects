use std::sync::Mutex;
static TOTAL: Mutex<i64> = Mutex::new(0);
const INVOICE: f64 = 19.75;
fn post(amount: f64) {
    let whole = amount as i64;
    let mut total = TOTAL.lock().unwrap();
    *total += whole;
}
fn main() {
    post(INVOICE);
    println!("total {}", TOTAL.lock().unwrap());
}
