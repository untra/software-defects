fn main() {
    let mut source = String::from("éab");
    let mut work = source.clone();
    let result = work.remove(0);
    assert!(result == 'é' && work.as_str() == "ab");
    assert!(source == "éab", "source changed");
    println!("ok");
}
