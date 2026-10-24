use std::{collections::HashMap, fs};
const CATALOG_NAME: &str = "café";
fn main() {
    let path = std::env::temp_dir().join(format!("catalog-{}.txt", std::process::id()));
    let latin1: Vec<u8> = CATALOG_NAME.chars().map(|letter| letter as u8).collect();
    fs::write(&path, latin1).unwrap();
    let bytes = fs::read(&path).unwrap();
    let name = String::from_utf8_lossy(&bytes).into_owned();
    let catalog = HashMap::from([("coffee", name)]);
    println!("name {}", catalog["coffee"]);
    fs::remove_file(path).unwrap();
}
