use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
static RATE: AtomicU32 = AtomicU32::new(2);
const WEIGHT: u32 = 5;
const NEW_RATE: u32 = 3;
fn shipping(weight: u32) -> u32 { weight * RATE.load(Ordering::Relaxed) }
fn quote(cache: &mut HashMap<u32, u32>, weight: u32) -> u32 {
    *cache.entry(weight).or_insert_with(|| shipping(weight))
}
fn main() {
    let mut cache = HashMap::new();
    quote(&mut cache, WEIGHT);
    RATE.store(NEW_RATE, Ordering::Relaxed);
    println!("shipping {}", quote(&mut cache, WEIGHT));
}
