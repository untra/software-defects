use std::collections::HashMap;

struct PriceService {
    db: HashMap<&'static str, u32>,
    cache: HashMap<&'static str, u32>,
}

impl PriceService {
    fn price(&mut self, sku: &'static str) -> u32 {
        if let Some(p) = self.cache.get(sku) {
            return *p;
        }
        let p = self.db[sku];
        self.cache.insert(sku, p);
        p
    }

    fn set_price(&mut self, sku: &'static str, cents: u32) {
        self.db.insert(sku, cents);
        self.cache.remove(sku);
    }
}

fn main() {
    let mut prices = PriceService {
        db: HashMap::from([("boots", 12000)]),
        cache: HashMap::new(),
    };
    prices.price("boots");
    prices.set_price("boots", 8000);
    println!(
        "checkout charges {} cents (sale price 8000)",
        prices.price("boots")
    );
}
