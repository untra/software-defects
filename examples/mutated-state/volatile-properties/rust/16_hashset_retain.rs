use std::collections::HashSet;

fn main() {
    let mut source = HashSet::from([1, 2]);
    let mut work = &mut source;
    work.retain(|x| *x == 2);
    assert!(work.len() == 1 && work.contains(&2));
    assert!(source == HashSet::from([1, 2]), "source changed");
    println!("ok");
}
