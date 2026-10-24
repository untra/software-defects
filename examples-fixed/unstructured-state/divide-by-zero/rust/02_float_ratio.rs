fn main() {
    let voltage = 12.0;
    let resistance = 0.0;
    if resistance == 0.0 {
        println!("zero divisor");
        return;
    }
    let current = voltage / resistance;
    println!("current {current}");
}
