fn main() {
    let price: u32 = std::hint::black_box(1_000_000_000);
    let qty: u32 = std::hint::black_box(5);
    let total = price * qty;
    println!("total {total}");
}
