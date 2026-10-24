use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
#[derive(Clone)]
struct Packing { notes: Rc<RefCell<Vec<Box<dyn Any>>>> }
struct Note { sku: &'static str }
impl Packing {
    fn stash(&self, value: Box<dyn Any>) { self.notes.borrow_mut().push(value); }
}
fn main() {
    let original = Packing { notes: Rc::new(RefCell::new(Vec::new())) };
    let next = original.clone();
    original.stash(Box::new(Note { sku: "pen" }));
    original.stash(Box::new(String::from("ink")));
    let notes = next.notes.borrow();
    let labels: Vec<_> = notes.iter().map(|note| note.downcast_ref::<Note>().unwrap().sku).collect();
    println!("labels {:?}", labels);
}
