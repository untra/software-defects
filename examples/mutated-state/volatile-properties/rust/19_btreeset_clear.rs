use std::collections::BTreeSet;

fn main() {
    let mut source = BTreeSet::from([1, 2]);
    let mut work = &mut source;
    work.clear();
    assert!(work.is_empty());
    assert!(source == BTreeSet::from([1, 2]), "source changed");
    println!("ok");
}
