const REQUESTED: &str = "pad";
const CATALOG: [(&str, u32); 1] = [("pen", 150)];
fn lookup(sku: &str) -> Option<u32> {
    CATALOG.iter().find(|(name, _)| *name == sku).map(|(_, cents)| *cents)
}
fn main() {
    let selected = lookup(REQUESTED);
    println!("cents {}", selected.unwrap());
}
