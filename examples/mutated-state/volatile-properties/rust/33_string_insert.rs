fn main() {
    let mut source = String::from("éab");
    let mut work = &mut source;
    work.insert(2, '!');
    assert!(work.as_str() == "é!ab");
    assert!(source == "éab", "source changed");
    println!("ok");
}
