const UNIT_PRICE: i32 = 10;

fn charge(quantity: i32) -> Result<i32, &'static str> {
    if quantity < 0 {
        return Err("quantity rejected");
    }
    Ok(quantity * UNIT_PRICE)
}

fn main() {
    match charge(-3) {
        Ok(total) => println!("charge {total}"),
        Err(error) => println!("{error}"),
    }
}
