use std::any::Any;
const BODY: &str = r#"{"reading":"10"}"#;
const INITIAL_READING: f64 = 0.0;
struct Meter { reading: Box<dyn Any> }
impl Meter {
    fn load(&mut self, body: &str) {
        let record = body.trim().strip_prefix('{').unwrap().strip_suffix('}').unwrap();
        let (key, value) = record.split_once(':').unwrap();
        if key.trim() == "\"reading\"" {
            self.reading = if value.trim().starts_with('"') {
                Box::new(value.trim().trim_matches('"').to_owned())
            } else {
                Box::new(value.trim().parse::<f64>().unwrap())
            };
        }
    }
    fn display(&self) -> String { format!("{:.1}", self.reading.downcast_ref::<f64>().unwrap()) }
}
fn main() {
    let mut meter = Meter { reading: Box::new(INITIAL_READING) };
    meter.load(BODY);
    println!("reading {}", meter.display());
}
