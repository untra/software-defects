use std::collections::{HashMap, HashSet};
const ATTEMPTS: usize = 32;
const TAGS: [&str; 8] = ["tea", "coffee", "milk", "bread", "rice", "beans", "pears", "plums"];
fn key() -> String {
    let tags: HashSet<_> = TAGS.into_iter().collect();
    tags.into_iter().collect::<Vec<_>>().join(",")
}
fn main() {
    let mut cache = HashMap::new();
    cache.insert(key(), "catalog");
    let misses = (0..ATTEMPTS).filter(|_| !cache.contains_key(&key())).count();
    println!("catalog misses {}", misses > 0);
}
