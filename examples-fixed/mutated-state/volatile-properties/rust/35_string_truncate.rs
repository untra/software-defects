fn main() {
    let mut source = String::from("éab");
    let mut work = source.clone();
    work.truncate(2);
    assert!(work.as_str() == "é");
    assert!(source == "éab", "source changed");
    println!("ok");
}
