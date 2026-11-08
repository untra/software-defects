const INITIAL_RATE: f64 = 1.0;
const RATE_INPUT: &str = "";
const RATE_LABELS: [(f64, &str); 3] = [(0.0, "standard"), (1.0, "express"), (2.0, "priority")];
fn quote(input: &str) -> String {
    let mut rate = INITIAL_RATE;
    rate = input.parse::<f64>().unwrap_or(f64::NAN);
    let tier = rate.round();
    RATE_LABELS.iter().find(|(key, _)| *key == tier).unwrap().1.to_uppercase()
}
fn main() { println!("shipping {}", quote(RATE_INPUT)); }
