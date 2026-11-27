fn main() {
    let jobs = 10;
    let buckets = std::hint::black_box(0);
    let remainder = jobs % buckets;
    println!("remainder {remainder}");
}
