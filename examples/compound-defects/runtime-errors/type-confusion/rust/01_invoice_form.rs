use std::any::Any;
const STARTING_CENTS: u32 = 1250;
const FORM_CENTS: &str = "1425";
struct Invoice { cents: Box<dyn Any> }
impl Invoice {
    fn apply(&mut self, cents: &str) { self.cents = Box::new(cents.to_owned()); }
    fn display(&self) -> String { self.cents.downcast_ref::<u32>().unwrap().to_string() }
}
fn main() {
    let mut invoice = Invoice { cents: Box::new(STARTING_CENTS) };
    invoice.apply(FORM_CENTS);
    println!("cents {}", invoice.display());
}
