fn main() {
    let mut source = String::from("éab");
    let mut work = &mut source;
    work.push_str("!");
    assert!(work.as_str() == "éab!");
    assert!(source == "éab", "source changed");
    println!("ok");
}
