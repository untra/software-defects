const MAX_COPIES: usize = 4;
const MAX_INPUT_ITEMS: usize = 4;
const MAX_OUTPUT_ITEMS: usize = 6;

fn expand<'a>(items: &[&'a str], copies: usize) -> Result<Vec<&'a str>, &'static str> {
    if copies > MAX_COPIES || items.len() > MAX_INPUT_ITEMS {
        return Err("input rejected");
    }
    let mut output = Vec::new();
    for _ in 0..copies {
        output.extend_from_slice(items);
    }
    Ok(output)
}

fn main() {
    match expand(&["a", "b", "c"], 3) {
        Ok(output) => println!("expanded {}", output.len()),
        Err(error) => println!("{error}"),
    }
}
