fn main() {
    let mut source = String::from("éab");
    let mut work = source.clone();
    work.clear();
    assert!(work.is_empty());
    assert!(source == "éab", "source changed");
    println!("ok");
}
