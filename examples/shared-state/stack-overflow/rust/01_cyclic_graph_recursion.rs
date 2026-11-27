use std::cell::RefCell;
use std::rc::Rc;

struct Folder {
    name: String,
    children: Vec<Rc<RefCell<Folder>>>,
}

fn total_folders(folder: &Rc<RefCell<Folder>>) -> usize {
    1 + folder.borrow().children.iter().map(total_folders).sum::<usize>()
}

fn main() {
    let root = Rc::new(RefCell::new(Folder { name: "root".into(), children: vec![] }));
    let shortcut = Rc::new(RefCell::new(Folder { name: "shortcut".into(), children: vec![Rc::clone(&root)] }));
    root.borrow_mut().children.push(shortcut);
    println!("{} folders under {}", total_folders(&root), root.borrow().name);
}
