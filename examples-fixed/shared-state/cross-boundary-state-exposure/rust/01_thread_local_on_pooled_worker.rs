use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;

thread_local! {
    static CART: RefCell<HashMap<String, Vec<String>>> = RefCell::new(HashMap::new());
}

fn handle_add_to_cart(user: &str, item: &str) -> Vec<String> {
    CART.with(|cart| {
        let mut carts = cart.borrow_mut();
        let items = carts.entry(user.to_string()).or_default();
        items.push(item.to_string());
        println!("{} sees cart {:?}", user, items);
        items.clone()
    })
}

fn main() {
    let (jobs, inbox) = mpsc::channel::<(String, String)>();
    let worker = thread::spawn(move || {
        for (user, item) in inbox {
            handle_add_to_cart(&user, &item);
        }
    });
    jobs.send(("alice".into(), "engagement ring".into()))
        .unwrap();
    jobs.send(("bob".into(), "socks".into())).unwrap();
    drop(jobs);
    worker.join().unwrap();
}
