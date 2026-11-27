use std::collections::HashMap;

struct Shipping {
    addresses: HashMap<u32, &'static str>,
    quotes: HashMap<u32, u32>,
}

impl Shipping {
    fn quote(&mut self, user: u32) -> u32 {
        if let Some(q) = self.quotes.get(&user) {
            return *q;
        }
        let q = if self.addresses[&user] == "US" { 500 } else { 2500 };
        self.quotes.insert(user, q);
        q
    }
}

fn main() {
    let mut shipping = Shipping { addresses: HashMap::from([(7, "US")]), quotes: HashMap::new() };
    shipping.quote(7);
    shipping.addresses.insert(7, "JP");
    println!("shipping to JP quoted at {} cents", shipping.quote(7));
}
