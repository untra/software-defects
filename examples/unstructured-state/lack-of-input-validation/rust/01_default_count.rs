fn main() {
    let raw = "12abc";
    let shipped: i32 = raw.parse().unwrap_or(0);
    println!("shipped {shipped}");
}
