use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const WATCHDOG: Duration = Duration::from_millis(200);
fn main() {
    let (done, result) = std::sync::mpsc::channel();
    thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread().enable_time().build().unwrap().block_on(async {
            let state = Arc::new(Mutex::new(0));
            let first = async {
                tokio::time::sleep(Duration::from_millis(1)).await;
                let mut guard = state.lock().unwrap();
                *guard += 1;
            };
            let second = async { *state.lock().unwrap() += 1; };
            let mut first = std::pin::pin!(first);
            let mut second = std::pin::pin!(second);
            let mut finished = [false, false];
            std::future::poll_fn(|cx| {
                use std::future::Future;
                if !finished[0] { finished[0] = first.as_mut().poll(cx).is_ready(); }
                if !finished[1] { finished[1] = second.as_mut().poll(cx).is_ready(); }
                if finished.iter().all(|done| *done) { std::task::Poll::Ready(()) }
                else { std::task::Poll::Pending }
            }).await;
            done.send(*state.lock().unwrap()).unwrap();
        });
    });
    match result.recv_timeout(WATCHDOG) {
        Ok(count) => println!("count: {count}"),
        Err(_) => println!("blocked"),
    }
}
