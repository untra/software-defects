const HISTORY_LENGTH: usize = 1_000_000;

struct Edit {
    _text: String,
    previous: Option<Box<Edit>>,
}

fn main() {
    let mut history: Option<Box<Edit>> = None;
    for i in 0..HISTORY_LENGTH {
        history = Some(Box::new(Edit { _text: format!("edit {}", i), previous: history }));
    }
    println!("recorded {} edits", HISTORY_LENGTH);
    drop(history);
    println!("history cleared");
}
