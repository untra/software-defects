fn main() {
    let total = 1200;
    let diners = std::hint::black_box(0);
    let each = total / diners;
    println!("each {each}");
}
