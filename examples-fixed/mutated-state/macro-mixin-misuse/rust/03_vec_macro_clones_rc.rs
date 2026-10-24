use std::cell::RefCell;
use std::rc::Rc;

const LANES: usize = 3;

fn main() {
    let lanes = (0..LANES)
        .map(|_| Rc::new(RefCell::new(Vec::<u32>::new())))
        .collect::<Vec<_>>();
    lanes[0].borrow_mut().push(42);
    for (i, lane) in lanes.iter().enumerate() {
        println!("lane {}: {:?}", i, lane.borrow());
    }
}
