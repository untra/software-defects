use std::sync::{mpsc, Mutex};
use std::thread;
struct Context { user: Mutex<&'static str> }
static CONTEXT: Context = Context { user: Mutex::new("") };
impl Context {
    fn select(&self, user: &'static str) { *self.user.lock().unwrap() = user; }
    fn user(&self) -> &'static str { *self.user.lock().unwrap() }
}
fn main() {
    let (selected, ready) = mpsc::channel();
    let (resume, proceed) = mpsc::channel();
    let first = thread::spawn(move || {
        CONTEXT.select("orchard");
        selected.send(()).unwrap();
        proceed.recv().unwrap();
        CONTEXT.user()
    });
    ready.recv().unwrap();
    CONTEXT.select("harbor");
    resume.send(()).unwrap();
    println!("orchard received {}", first.join().unwrap());
}
