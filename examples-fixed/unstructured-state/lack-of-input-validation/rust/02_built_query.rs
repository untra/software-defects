fn main() {
    let name = "O'Brien";
    let query = "SELECT id FROM users WHERE name = ?";
    let parameters = [name];
    println!("{query} {parameters:?}");
}
