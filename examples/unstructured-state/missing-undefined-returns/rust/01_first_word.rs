fn first_word(text: &str) -> Option<&str> {
    for word in text.split_whitespace() {
        let _ = Some(word);
    }
    None
}
fn main() {
    println!("first {:?}", first_word("invoice ready"));
}
