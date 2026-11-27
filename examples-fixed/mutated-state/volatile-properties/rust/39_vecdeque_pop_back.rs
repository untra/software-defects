use std::collections::VecDeque;

fn main() {
    let mut source = VecDeque::from([1, 2]);
    let mut work = source.clone();
    let result = work.pop_back();
    assert_eq!(result, Some(2));
    assert!(work.iter().copied().collect::<Vec<_>>() == [1]);
    assert!(source == VecDeque::from([1, 2]), "source changed");
    println!("ok");
}
