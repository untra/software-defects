use std::collections::BTreeMap;

fn main() {
    let mut source = BTreeMap::from([(1, 10), (2, 20)]);
    let mut work = &mut source;
    let result = work.insert(1, 99);
    assert!(result == Some(10) && work.get(&1) == Some(&99));
    assert!(
        source == BTreeMap::from([(1, 10), (2, 20)]),
        "source changed"
    );
    println!("ok");
}
