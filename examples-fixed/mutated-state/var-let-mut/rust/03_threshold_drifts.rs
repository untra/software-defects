const THRESHOLD: i32 = 10;
const SCALE: i32 = 2;

fn process_signals(signals: &[i32]) -> Vec<i32> {
    let mut results = Vec::new();
    let threshold = THRESHOLD;
    for &signal in signals {
        if signal > threshold {
            results.push(signal * SCALE);
        }
    }
    results
}

fn main() {
    println!("{:?}", process_signals(&[20, 15, 30]));
}
