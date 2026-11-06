use std::collections::BTreeMap;

fn main() {
    let mut source = BTreeMap::from([(1, 10), (2, 20)]);
    let mut work = source.clone();
    let entry = work.entry(3);
    let result = entry.or_insert(30);
    assert_eq!(*result, 30);
    assert!(work.get(&3) == Some(&30));
    assert!(
        source == BTreeMap::from([(1, 10), (2, 20)]),
        "source changed"
    );
    println!("ok");
}
