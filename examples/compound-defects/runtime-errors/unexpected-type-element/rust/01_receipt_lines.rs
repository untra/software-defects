use std::any::Any;
const INITIAL_CENTS: u32 = 250;
const FORM_CENTS: &str = "275";
struct Line { cents: Box<dyn Any> }
impl Line {
    fn edit(&mut self, value: &str) { self.cents = Box::new(value.to_owned()); }
}
fn main() {
    let mut lines = vec![Line { cents: Box::new(INITIAL_CENTS) }];
    lines[0].edit(FORM_CENTS);
    let formatted: Vec<_> = lines.iter().map(|line| line.cents.downcast_ref::<u32>().unwrap().to_string()).collect();
    println!("receipt {:?}", formatted);
}
