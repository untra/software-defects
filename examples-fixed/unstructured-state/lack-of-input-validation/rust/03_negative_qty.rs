fn main() {
    let qty = -3;
    let price = 10;
    if qty < 0 {
        println!("quantity rejected");
        return;
    }
    println!("charge {}", qty * price);
}
