use std::cell::Cell;

struct Tokenizer {
    cursor: Cell<usize>,
    words: Vec<&'static str>,
}

impl Tokenizer {
    fn peek(&self) -> Option<&'static str> {
        let i = self.cursor.get();
        self.cursor.set(i + 1);
        self.words.get(i).copied()
    }
}

fn main() {
    let tokens = Tokenizer { cursor: Cell::new(0), words: vec!["let", "x", "="] };
    let first = tokens.peek();
    let again = tokens.peek();
    println!("first {:?}, again {:?}", first, again);
}
