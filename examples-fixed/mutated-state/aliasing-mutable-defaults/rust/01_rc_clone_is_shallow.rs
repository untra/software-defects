use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
struct Playlist {
    name: String,
    tracks: Rc<RefCell<Vec<String>>>,
}

fn main() {
    let original = Playlist {
        name: "road trip".to_string(),
        tracks: Rc::new(RefCell::new(vec!["song a".to_string()])),
    };
    let mut remix = original.clone();
    remix.tracks = Rc::new(RefCell::new(original.tracks.borrow().clone()));
    remix.name = "road trip remix".to_string();
    remix.tracks.borrow_mut().push("remix b".to_string());
    println!("{}: {:?}", original.name, original.tracks.borrow());
}
