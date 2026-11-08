fn main() {
    let mut source = vec![1, 2, 3];
    let mut work = source.clone();
    let removed: Vec<_> = work.drain(1..).collect();
    assert_eq!(removed, vec![2, 3]);
    assert!(work.as_slice() == [1]);
    assert!(source == vec![1, 2, 3], "source changed");
    println!("ok");
}
