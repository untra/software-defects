use std::cell::RefCell;
use std::rc::Rc;

struct Folder {
    name: String,
    children: Vec<Rc<RefCell<Folder>>>,
}

fn total_folders(folder: &Rc<RefCell<Folder>>) -> usize {
    let mut seen = std::collections::HashSet::new();
    let mut pending = vec![Rc::clone(folder)];
    while let Some(current) = pending.pop() {
        if seen.insert(Rc::as_ptr(&current)) {
            pending.extend(current.borrow().children.iter().cloned());
        }
    }
    seen.len()
}

fn main() {
    let root = Rc::new(RefCell::new(Folder {
        name: "root".into(),
        children: vec![],
    }));
    let shortcut = Rc::new(RefCell::new(Folder {
        name: "shortcut".into(),
        children: vec![Rc::clone(&root)],
    }));
    root.borrow_mut().children.push(shortcut);
    println!(
        "{} folders under {}",
        total_folders(&root),
        root.borrow().name
    );
    root.borrow_mut().children.clear();
}
