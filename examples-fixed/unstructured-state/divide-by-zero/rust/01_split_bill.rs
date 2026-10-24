fn main() {
    let total = 1200;
    let diners = std::hint::black_box(0);
    if diners == 0 {
        println!("zero divisor");
        return;
    }
    let each = total / diners;
    println!("each {each}");
}
