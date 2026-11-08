use std::collections::HashSet;

fn main() {
    let mut source = HashSet::from([1, 2]);
    let mut work = &mut source;
    let result = work.insert(3);
    assert!(result && work.len() == 3 && work.contains(&3));
    assert!(source == HashSet::from([1, 2]), "source changed");
    println!("ok");
}
