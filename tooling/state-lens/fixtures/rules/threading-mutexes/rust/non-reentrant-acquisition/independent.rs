use std::sync::Mutex;
fn f(a: &Mutex<i32>, b: &Mutex<i32>) { let first = a.lock().unwrap(); let second = b.lock().unwrap(); }
