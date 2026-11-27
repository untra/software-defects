fn main() {
    let mut source = vec![3, 1, 1, 2];
    let mut work = source.clone();
    work.push(9);
    assert_eq!(work.as_slice(), &[3, 1, 1, 2, 9]);
    assert_eq!(source, vec![3, 1, 1, 2], "source changed");
    println!("ok");
}
