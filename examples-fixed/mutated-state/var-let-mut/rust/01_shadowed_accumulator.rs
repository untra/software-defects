fn main() {
    let prices = [1200, 450, 3000];
    let mut total = 0;
    for price in prices {
        total += price;
        println!("running total: {}", total);
    }
    println!("final total: {}", total);
}
