const HISTORY_LENGTH: usize = 1_000_000;

struct Edit {
    _text: String,
    previous: Option<Box<Edit>>,
}

impl Drop for Edit {
    fn drop(&mut self) {
        let mut previous = self.previous.take();
        while let Some(mut edit) = previous {
            previous = edit.previous.take();
        }
    }
}

fn main() {
    let mut history: Option<Box<Edit>> = None;
    for i in 0..HISTORY_LENGTH {
        history = Some(Box::new(Edit {
            _text: format!("edit {}", i),
            previous: history,
        }));
    }
    println!("recorded {} edits", HISTORY_LENGTH);
    drop(history);
    println!("history cleared");
}
