use std::sync::Mutex;
fn f(m: &Mutex<i32>) { let first = m.lock().unwrap(); drop(first); let second = m.lock().unwrap(); }
