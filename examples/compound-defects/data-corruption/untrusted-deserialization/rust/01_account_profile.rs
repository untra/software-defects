const BODY: &str = r#"{"name":"mira","admin":true}"#;
struct Account { name: String, admin: bool }
fn receive(account: &mut Account, body: &str) {
    for field in body.trim().trim_start_matches('{').trim_end_matches('}').split(',') {
        let (key, value) = field.split_once(':').unwrap();
        match key.trim().trim_matches('"') {
            "name" => account.name = value.trim().trim_matches('"').to_owned(),
            "admin" => account.admin = value.trim().parse::<bool>().unwrap(),
            _ => {}
        }
    }
}
fn main() {
    let mut account = Account { name: String::from("mira"), admin: false };
    receive(&mut account, BODY);
    println!("admin {}", account.admin);
}
