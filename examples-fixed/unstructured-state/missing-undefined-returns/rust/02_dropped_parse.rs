fn qty_from(raw: &str) -> Result<i32, std::num::ParseIntError> {
    raw.parse::<i32>()
}

fn main() {
    println!("qty {:?}", qty_from("nope"));
}
