fn main() {
    let mut source = vec![3, 1, 1, 2];
    let mut work = source.clone();
    work.sort_by(|a, b| a.cmp(b));
    assert_eq!(work.as_slice(), &[1, 1, 2, 3]);
    assert_eq!(source, vec![3, 1, 1, 2], "source changed");
    println!("ok");
}
