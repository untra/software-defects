use std::sync::Mutex;
use std::thread;
use std::time::Duration;

const RENDER_LATENCY: Duration = Duration::from_millis(20);
const REQUEST_STAGGER: Duration = Duration::from_millis(5);

static CURRENT_USER: Mutex<Option<String>> = Mutex::new(None);

fn handle_dashboard(user: &str) -> String {
    thread::sleep(RENDER_LATENCY);
    let viewer = user;
    format!("{} received account data of {}", user, viewer)
}

fn main() {
    let alice = thread::spawn(|| handle_dashboard("alice"));
    thread::sleep(REQUEST_STAGGER);
    let bob = thread::spawn(|| handle_dashboard("bob"));
    println!("{}", alice.join().unwrap());
    println!("{}", bob.join().unwrap());
}
