use std::collections::HashMap;

fn main() {
    let env: HashMap<&str, &str> = HashMap::from([("DB_HOST", "db.internal")]);
    let Some(port) = env
        .get("DB_PORT")
        .and_then(|p| p.parse::<u16>().ok())
        .filter(|port| *port != 0)
    else {
        println!("DB_PORT is required and must be a valid nonzero port");
        return;
    };
    println!("connecting to {}:{}", env["DB_HOST"], port);
}
