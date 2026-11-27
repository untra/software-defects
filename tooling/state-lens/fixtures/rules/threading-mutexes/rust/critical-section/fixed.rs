use std::sync::Mutex;
fn f(m: &Mutex<i32>) { let guard = m.lock().unwrap(); }
