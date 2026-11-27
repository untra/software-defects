use std::collections::HashSet;

fn main() {
    let mut source = HashSet::from([1, 2]);
    let mut work = &mut source;
    let result = work.remove(&1);
    assert!(result && !work.contains(&1));
    assert!(source == HashSet::from([1, 2]), "source changed");
    println!("ok");
}
