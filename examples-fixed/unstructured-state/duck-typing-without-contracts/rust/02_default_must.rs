trait Catalog {
    fn find(&self, sku: &str) -> Option<&str>;

    fn must(&self, sku: &str) -> Option<&str> {
        self.find(sku)
    }
}

struct Empty;

impl Catalog for Empty {
    fn find(&self, _sku: &str) -> Option<&str> {
        None
    }
}

fn main() {
    let catalog = Empty;
    println!("sku {:?}", catalog.must("bolt"));
}
