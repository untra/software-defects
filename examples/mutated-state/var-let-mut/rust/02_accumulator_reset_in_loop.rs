fn main() {
    let orders = vec![vec![2, 3], vec![5], vec![1, 1, 1]];
    let mut items_shipped = 0;
    for order in &orders {
        items_shipped = 0;
        for qty in order {
            items_shipped += qty;
        }
    }
    println!("items shipped: {}", items_shipped);
}
