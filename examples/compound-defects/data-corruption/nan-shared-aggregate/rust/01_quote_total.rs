use std::sync::Mutex;
static TOTAL: Mutex<f64> = Mutex::new(100.0);
const EMPTY_AMOUNT: f64 = 0.0;
const EMPTY_UNITS: f64 = 0.0;
fn post(amount: f64, units: f64) {
    let quote = amount / units;
    let mut total = TOTAL.lock().unwrap();
    *total += quote;
}
fn main() {
    post(EMPTY_AMOUNT, EMPTY_UNITS);
    println!("total {}", TOTAL.lock().unwrap());
}
