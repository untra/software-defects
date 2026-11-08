use std::collections::HashMap;

fn main() {
    let mut source = HashMap::from([(1, 10), (2, 20)]);
    let mut work = source.clone();
    let entry = work.entry(1);
    entry.and_modify(|value| *value = 99);
    assert!(work.get(&1) == Some(&99));
    assert!(
        source == HashMap::from([(1, 10), (2, 20)]),
        "source changed"
    );
    println!("ok");
}
