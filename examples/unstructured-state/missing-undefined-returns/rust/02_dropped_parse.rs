fn qty_from(raw: &str) -> i32 {
    let parsed = raw.parse::<i32>();
    let _ = parsed;
    0
}

fn main() {
    println!("qty {}", qty_from("nope"));
}
