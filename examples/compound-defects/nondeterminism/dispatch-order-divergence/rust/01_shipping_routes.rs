use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
macro_rules! install {
    ($routes:expr, $handler:expr) => { $routes.lock().unwrap().insert("shipping", $handler) };
}
fn main() {
    let routes = Arc::new(Mutex::new(HashMap::new()));
    let (release, ready) = mpsc::channel();
    let earlier_routes = routes.clone();
    let earlier = thread::spawn(move || {
        ready.recv().unwrap();
        install!(earlier_routes, "standard");
    });
    let later_routes = routes.clone();
    thread::spawn(move || { install!(later_routes, "express"); }).join().unwrap();
    release.send(()).unwrap();
    earlier.join().unwrap();
    println!("shipping {}", routes.lock().unwrap()["shipping"]);
}
