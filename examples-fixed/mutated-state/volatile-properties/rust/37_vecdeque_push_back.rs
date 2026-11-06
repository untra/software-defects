use std::collections::VecDeque;

fn main() {
    let mut source = VecDeque::from([1, 2]);
    let mut work = source.clone();
    work.push_back(3);
    assert!(work.iter().copied().collect::<Vec<_>>() == [1, 2, 3]);
    assert!(source == VecDeque::from([1, 2]), "source changed");
    println!("ok");
}
