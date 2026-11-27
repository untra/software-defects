use std::cell::RefCell;
use std::rc::Rc;

const LANES: usize = 3;

fn main() {
    let lanes = vec![Rc::new(RefCell::new(Vec::<u32>::new())); LANES];
    lanes[0].borrow_mut().push(42);
    for (i, lane) in lanes.iter().enumerate() {
        println!("lane {}: {:?}", i, lane.borrow());
    }
}
