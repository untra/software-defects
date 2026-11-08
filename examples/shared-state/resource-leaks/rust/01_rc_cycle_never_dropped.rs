use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

static DROPPED: AtomicUsize = AtomicUsize::new(0);

struct Node {
    name: &'static str,
    peer: RefCell<Option<Rc<Node>>>,
}

impl Drop for Node {
    fn drop(&mut self) {
        DROPPED.fetch_add(1, Ordering::SeqCst);
    }
}

fn open_session() {
    let client = Rc::new(Node { name: "client", peer: RefCell::new(None) });
    let server = Rc::new(Node { name: "server", peer: RefCell::new(None) });
    *client.peer.borrow_mut() = Some(Rc::clone(&server));
    *server.peer.borrow_mut() = Some(Rc::clone(&client));
    println!("session {} <-> {} open", client.name, server.name);
}

fn main() {
    open_session();
    println!("nodes freed after session ended: {} of 2", DROPPED.load(Ordering::SeqCst));
}
