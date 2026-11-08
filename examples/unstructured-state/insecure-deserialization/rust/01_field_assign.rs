fn apply(line: &str) -> (String, bool) {
    let mut name = String::from("ada");
    let mut admin = false;
    for part in line.split(',') {
        let mut pieces = part.splitn(2, '=');
        let key = pieces.next().unwrap_or("");
        let value = pieces.next().unwrap_or("");
        match key {
            "name" => name = value.to_string(),
            "admin" => admin = value == "true",
            _ => {}
        }
    }
    (name, admin)
}

fn main() {
    let (name, admin) = apply("name=ada,admin=true");
    println!("name={name} admin={admin}");
}
