use std::{collections::HashMap, fs::{self, OpenOptions}, io::Write, path::Path};
const WAREHOUSE: &str = r#"{"version":2,"weightGrams":500}"#;
const KIOSK: &str = r#"{"weight":2}"#;
fn append(path: &Path, body: &str) {
    writeln!(OpenOptions::new().create(true).append(true).open(path).unwrap(), "{body}").unwrap();
}
fn read_record(body: &str) -> HashMap<&str, &str> {
    body.trim().trim_start_matches('{').trim_end_matches('}').split(',').map(|field| {
        let (key, value) = field.split_once(':').unwrap();
        (key.trim().trim_matches('"'), value.trim())
    }).collect()
}
fn main() {
    let path = std::env::temp_dir().join(format!("shipments-{}.jsonl", std::process::id()));
    append(&path, WAREHOUSE);
    append(&path, KIOSK);
    for line in fs::read_to_string(&path).unwrap().lines() {
        let record = read_record(line);
        println!("grams {}", record.get("weightGrams").unwrap_or(&"0"));
    }
    fs::remove_file(path).unwrap();
}
