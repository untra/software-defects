fn main() {
    let mut source = vec![3, 1, 1, 2];
    let mut work = source.clone();
    work.insert(1, 9);
    assert_eq!(work.as_slice(), &[3, 9, 1, 1, 2]);
    assert_eq!(source, vec![3, 1, 1, 2], "source changed");
    println!("ok");
}
