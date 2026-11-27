const MAX_LABEL_LENGTH: usize = 6;

fn main() {
    let requested = 8;
    if requested > MAX_LABEL_LENGTH {
        println!("label rejected");
        return;
    }
    let label = "x".repeat(requested);
    println!("label {label}");
}
