use std::collections::HashMap;
const ASSIGNED_SEAT: &str = "b12";
const RESERVED: &str = "reserved";
const RUSH: &str = "rush";
fn issue(kind: &str) -> HashMap<&str, &str> {
    let mut ticket = HashMap::new();
    ticket.insert("kind", kind);
    if kind == RESERVED { ticket.insert("seat", ASSIGNED_SEAT); }
    ticket
}
fn print_seat(ticket: &HashMap<&str, &str>) -> String { ticket["seat"].to_uppercase() }
fn main() {
    println!("seat {}", print_seat(&issue(RUSH)));
}
