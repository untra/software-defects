use std::fs;
const CUSTOMER: &str = "O'Neil";
fn main() {
    let path = std::env::temp_dir().join(format!("customers-{}.txt", std::process::id()));
    fs::write(&path, CUSTOMER).unwrap();
    let name = fs::read_to_string(&path).unwrap();
    let query = "SELECT id FROM customers WHERE name = '".to_owned() + &name + "'";
    println!("{query}");
    fs::remove_file(path).unwrap();
}
