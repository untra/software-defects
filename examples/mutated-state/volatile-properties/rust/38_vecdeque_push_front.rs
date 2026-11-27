use std::collections::VecDeque;

fn main() {
    let mut source = VecDeque::from([1, 2]);
    let mut work = &mut source;
    work.push_front(3);
    assert!(work.iter().copied().collect::<Vec<_>>() == [3, 1, 2]);
    assert!(source == VecDeque::from([1, 2]), "source changed");
    println!("ok");
}
