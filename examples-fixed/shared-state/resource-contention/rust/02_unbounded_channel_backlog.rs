use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

const EVENTS: usize = 10_000;
const HANDLE_COST: Duration = Duration::from_millis(1);
const OBSERVE_AFTER: Duration = Duration::from_millis(50);
const BACKLOG_ALARM: usize = 1_000;

fn main() {
    let (events, inbox) = mpsc::sync_channel::<usize>(BACKLOG_ALARM);
    let handled = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&handled);
    let worker = thread::spawn(move || {
        for _ in inbox {
            thread::sleep(HANDLE_COST);
            counter.fetch_add(1, Ordering::SeqCst);
        }
    });
    for e in 0..EVENTS {
        events.send(e).unwrap();
    }
    drop(events);
    worker.join().unwrap();
    thread::sleep(OBSERVE_AFTER);
    let backlog = EVENTS - handled.load(Ordering::SeqCst);
    println!(
        "backlog over {} events: {}",
        BACKLOG_ALARM,
        backlog > BACKLOG_ALARM
    );
}
