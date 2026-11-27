use std::collections::BTreeMap;

fn main() {
    let mut source = BTreeMap::from([(1, 10), (2, 20)]);
    let mut work = &mut source;
    let result = work.remove(&1);
    assert!(result == Some(10) && !work.contains_key(&1));
    assert!(
        source == BTreeMap::from([(1, 10), (2, 20)]),
        "source changed"
    );
    println!("ok");
}
