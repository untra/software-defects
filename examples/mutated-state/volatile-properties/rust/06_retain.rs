fn main() {
    let mut source = vec![3, 1, 1, 2];
    let mut work = &mut source;
    work.retain(|value| *value > 1);
    assert_eq!(work.as_slice(), &[3, 2]);
    assert_eq!(source, vec![3, 1, 1, 2], "source changed");
    println!("ok");
}
