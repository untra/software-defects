const UNIT_PRICE: i32 = 10;

fn charge(quantity: i32) -> i32 {
    quantity * UNIT_PRICE
}

fn main() {
    println!("charge {}", charge(-3));
}
