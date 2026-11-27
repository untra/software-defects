use std::collections::HashMap;

fn main() {
    let env: HashMap<&str, &str> = HashMap::from([("DB_HOST", "db.internal")]);
    let port: u16 = env
        .get("DB_PORT")
        .and_then(|p| p.parse().ok())
        .unwrap_or_default();
    println!("connecting to {}:{}", env["DB_HOST"], port);
}
