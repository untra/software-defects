use std::collections::HashSet;

fn main() {
    let mut source = HashSet::from([1, 2]);
    let mut work = source.clone();
    work.clear();
    assert!(work.is_empty());
    assert!(source == HashSet::from([1, 2]), "source changed");
    println!("ok");
}
