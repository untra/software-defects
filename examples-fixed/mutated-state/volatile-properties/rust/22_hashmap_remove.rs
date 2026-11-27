use std::collections::HashMap;

fn main() {
    let mut source = HashMap::from([(1, 10), (2, 20)]);
    let mut work = source.clone();
    let result = work.remove(&1);
    assert!(result == Some(10) && !work.contains_key(&1));
    assert!(
        source == HashMap::from([(1, 10), (2, 20)]),
        "source changed"
    );
    println!("ok");
}
