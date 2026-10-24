fn main() {
    let price: u32 = std::hint::black_box(1_000_000_000);
    let qty: u32 = std::hint::black_box(5);
    let Some(total) = price.checked_mul(qty) else {
        println!("line rejected");
        return;
    };
    println!("total {total}");
}
