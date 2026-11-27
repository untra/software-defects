const CENTS_PER_DOLLAR: i32 = 100;

fn parse_amount(raw: &str, scales_to_cents: bool) -> i32 {
    let dollars: i32 = raw.parse().unwrap();
    if scales_to_cents {
        dollars * CENTS_PER_DOLLAR
    } else {
        dollars
    }
}

fn main() {
    println!("amount {}", parse_amount("10", false));
}
