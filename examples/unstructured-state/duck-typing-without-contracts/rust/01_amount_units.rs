trait Charger {
    fn amount(&self) -> i32;
}

struct Card {
    cents: i32,
}

impl Charger for Card {
    fn amount(&self) -> i32 {
        self.cents
    }
}

struct Invoice {
    dollars: i32,
}

impl Charger for Invoice {
    fn amount(&self) -> i32 {
        self.dollars
    }
}

fn charge(source: &dyn Charger) -> i32 {
    source.amount()
}

fn main() {
    println!("charged {}", charge(&Card { cents: 250 }));
    println!("charged {}", charge(&Invoice { dollars: 250 }));
}
