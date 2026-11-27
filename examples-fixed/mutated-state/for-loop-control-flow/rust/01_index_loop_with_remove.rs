fn main() {
    let mut readings = vec![-1, -2, 5, -3, 7];
    let mut i = 0;
    while i < readings.len() {
        if readings[i] < 0 {
            readings.remove(i);
            continue;
        }
        i += 1;
    }
    println!("{:?}", readings);
}
