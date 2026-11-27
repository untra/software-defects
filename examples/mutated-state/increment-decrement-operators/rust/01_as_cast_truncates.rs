fn main() {
    let item_count: usize = 300;
    let header_byte = item_count as u8;
    println!("items encoded in header: {}", header_byte);
}
