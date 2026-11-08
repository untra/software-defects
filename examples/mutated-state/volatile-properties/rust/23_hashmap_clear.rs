use std::collections::HashMap;

fn main() {
    let mut source = HashMap::from([(1, 10), (2, 20)]);
    let mut work = &mut source;
    work.clear();
    assert!(work.is_empty());
    assert!(
        source == HashMap::from([(1, 10), (2, 20)]),
        "source changed"
    );
    println!("ok");
}
