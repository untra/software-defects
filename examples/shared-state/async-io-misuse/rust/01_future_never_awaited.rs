use std::cell::RefCell;

async fn send_receipt(outbox: &RefCell<Vec<String>>, email: &str) {
    outbox.borrow_mut().push(format!("receipt to {}", email));
}

fn checkout(outbox: &RefCell<Vec<String>>, email: &str) -> &'static str {
    let _ = send_receipt(outbox, email);
    "order placed"
}

fn main() {
    let outbox = RefCell::new(Vec::new());
    println!("{}", checkout(&outbox, "ana@example.com"));
    println!("receipts sent: {}", outbox.borrow().len());
}
