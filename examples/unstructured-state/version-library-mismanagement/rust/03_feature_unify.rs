const SCALES_TO_CENTS: bool = true;

fn parse_amount(raw: &str) -> i32 {
    let dollars: i32 = raw.parse().unwrap();
    if SCALES_TO_CENTS {
        dollars * 100
    } else {
        dollars
    }
}

fn main() {
    println!("amount {}", parse_amount("10"));
}
