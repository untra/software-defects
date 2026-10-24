use std::cell::RefCell;
use std::sync::mpsc;
use std::thread;

thread_local! {
    static CART: RefCell<Vec<String>> = RefCell::new(Vec::new());
}

fn handle_add_to_cart(user: &str, item: &str) -> Vec<String> {
    CART.with(|cart| {
        cart.borrow_mut().push(item.to_string());
        println!("{} sees cart {:?}", user, cart.borrow());
        cart.borrow().clone()
    })
}

fn main() {
    let (jobs, inbox) = mpsc::channel::<(String, String)>();
    let worker = thread::spawn(move || {
        for (user, item) in inbox {
            handle_add_to_cart(&user, &item);
        }
    });
    jobs.send(("alice".into(), "engagement ring".into())).unwrap();
    jobs.send(("bob".into(), "socks".into())).unwrap();
    drop(jobs);
    worker.join().unwrap();
}
