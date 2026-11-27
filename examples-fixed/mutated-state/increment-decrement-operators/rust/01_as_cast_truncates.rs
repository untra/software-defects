fn main() {
    let item_count: usize = 300;
    let Ok(header_byte) = u8::try_from(item_count) else {
        println!("item count exceeds header capacity");
        return;
    };
    println!("items encoded in header: {}", header_byte);
}
