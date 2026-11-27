fn main() {
    let mut source = vec![1, 2, 3];
    let original_capacity = source.capacity();
    let mut work = &mut source;
    let additional = work.capacity() + 1;
    work.reserve(additional);
    assert!(work.capacity() >= work.len() + additional && work.as_slice() == [1, 2, 3]);
    assert!(source.capacity() == original_capacity, "source changed");
    println!("ok");
}
