fn main() {
    let mut source = vec![3, 1, 1, 2];
    let mut work = &mut source;
    work.reverse();
    assert_eq!(work.as_slice(), &[2, 1, 1, 3]);
    assert_eq!(source, vec![3, 1, 1, 2], "source changed");
    println!("ok");
}
