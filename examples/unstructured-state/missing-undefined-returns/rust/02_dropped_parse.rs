fn qty_from(raw: &str) -> Result<i32, std::num::ParseIntError> {
    match raw.parse::<i32>() {
        Ok(quantity) => Ok(quantity),
        Err(_) => Ok(0),
    }
}
fn main() {
    println!("qty {:?}", qty_from("nope"));
}
