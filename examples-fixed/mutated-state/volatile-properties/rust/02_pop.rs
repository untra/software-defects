fn main() {
    let mut source = vec![3, 1, 1, 2];
    let mut work = source.clone();
    let result = work.pop();
    assert_eq!(work.as_slice(), &[3, 1, 1]);
    assert_eq!(result, Some(2));
    assert_eq!(source, vec![3, 1, 1, 2], "source changed");
    println!("ok");
}
