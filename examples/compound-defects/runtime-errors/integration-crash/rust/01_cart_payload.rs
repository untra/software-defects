use std::sync::Mutex;
const INITIAL_QUANTITY: u32 = 2;
static CART: Mutex<Option<u32>> = Mutex::new(Some(INITIAL_QUANTITY));
struct Payload { qty: Option<u32> }
fn apply(payload: Payload) { *CART.lock().unwrap() = payload.qty; }
fn main() {
    let payload = Payload { qty: None };
    apply(payload);
    println!("quantity {}", CART.lock().unwrap().unwrap());
}
