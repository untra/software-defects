use std::collections::VecDeque;

fn main() {
    let mut source = VecDeque::from([1, 2]);
    let mut work = &mut source;
    let result = work.pop_front();
    assert_eq!(result, Some(1));
    assert!(work.iter().copied().collect::<Vec<_>>() == [2]);
    assert!(source == VecDeque::from([1, 2]), "source changed");
    println!("ok");
}
