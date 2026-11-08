use std::collections::BTreeSet;

fn main() {
    let mut source = BTreeSet::from([1, 2]);
    let mut work = source.clone();
    work.retain(|x| *x == 2);
    assert!(work.len() == 1 && work.contains(&2));
    assert!(source == BTreeSet::from([1, 2]), "source changed");
    println!("ok");
}
