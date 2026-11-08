use std::collections::BTreeSet;

fn main() {
    let mut source = BTreeSet::from([1, 2]);
    let mut work = &mut source;
    let result = work.remove(&1);
    assert!(result && !work.contains(&1));
    assert!(source == BTreeSet::from([1, 2]), "source changed");
    println!("ok");
}
