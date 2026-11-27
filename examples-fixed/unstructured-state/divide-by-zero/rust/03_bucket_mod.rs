fn main() {
    let jobs = 10;
    let buckets = std::hint::black_box(0);
    if buckets == 0 {
        println!("zero divisor");
        return;
    }
    let remainder = jobs % buckets;
    println!("remainder {remainder}");
}
