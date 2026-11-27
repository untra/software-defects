const ITEMS: [&str; 2] = ["pen", "pad"];
fn collect(items: &[&str]) -> Vec<String> {
    let mut labels = Vec::new();
    let mut index = 0;
    while index <= items.len() {
        labels.push(items[index].to_owned());
        index += 1;
    }
    labels
}
fn main() { println!("labels {:?}", collect(&ITEMS)); }
