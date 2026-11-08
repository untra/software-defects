use std::collections::HashMap;
const BODY: &str = r#"{"colour":"blue"}"#;
macro_rules! load {
    ($target:expr, $body:expr) => {
        for field in $body.trim().trim_start_matches('{').trim_end_matches('}').split(',') {
            let (key, value) = field.split_once(':').unwrap();
            $target.insert(key.trim().trim_matches('"'), value.trim().trim_matches('"'));
        }
    };
}
fn main() {
    let mut paint = HashMap::new();
    load!(paint, BODY);
    println!("paint {}", paint["color"].to_uppercase());
}
