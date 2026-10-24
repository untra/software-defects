const LIMIT: i32 = 10;

macro_rules! clamp_max {
    ($val:expr, $max:expr) => {
        if $val > $max { $max } else { $val }
    };
}

fn main() {
    let mut sensor = vec![5, 50, 7].into_iter();
    let mut next_reading = || sensor.next().unwrap_or(0);
    let first = clamp_max!(next_reading(), LIMIT);
    let second = clamp_max!(next_reading(), LIMIT);
    println!("first {} second {}", first, second);
}
