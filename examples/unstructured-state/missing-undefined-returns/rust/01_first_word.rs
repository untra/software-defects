fn first_word(text: &str) -> &str {
    text.split_whitespace().next().unwrap()
}

fn main() {
    println!("{}", first_word(""));
}
