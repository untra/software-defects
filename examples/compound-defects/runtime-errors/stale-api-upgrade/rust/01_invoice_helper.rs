use std::collections::HashMap;
const INVOICE_CENTS: u32 = 450;
type Methods = HashMap<&'static str, fn() -> u32>;
macro_rules! receipt {
    ($invoice:expr) => { $invoice["amount"]() };
}
fn total_cents() -> u32 { INVOICE_CENTS }
fn main() {
    let invoice: Methods = HashMap::from([("totalCents", total_cents as fn() -> u32)]);
    println!("receipt {}", receipt!(invoice));
}
