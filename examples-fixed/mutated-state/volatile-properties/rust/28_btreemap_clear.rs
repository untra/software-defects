use std::collections::BTreeMap;

fn main() {
    let mut source = BTreeMap::from([(1, 10), (2, 20)]);
    let mut work = source.clone();
    work.clear();
    assert!(work.is_empty());
    assert!(
        source == BTreeMap::from([(1, 10), (2, 20)]),
        "source changed"
    );
    println!("ok");
}
