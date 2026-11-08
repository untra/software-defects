fn first_word(text: &str) -> Option<&str> {
    text.split_whitespace().next()
}

fn main() {
    println!("{:?}", first_word(""));
}
