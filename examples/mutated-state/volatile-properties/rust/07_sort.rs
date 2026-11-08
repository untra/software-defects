fn main() {
    let mut source = vec![3, 1, 1, 2];
    let mut work = &mut source;
    work.sort();
    assert_eq!(work.as_slice(), &[1, 1, 2, 3]);
    assert_eq!(source, vec![3, 1, 1, 2], "source changed");
    println!("ok");
}
