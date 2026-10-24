use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const READ_TIMEOUT: Duration = Duration::from_millis(50);
const SLOW_FIRST_REPLY: Duration = Duration::from_millis(100);
const MAX_ATTEMPTS: usize = 3;
const SETTLE: Duration = Duration::from_millis(150);

fn place_order(addr: &str) -> std::io::Result<String> {
    let mut last_err = None;
    for _ in 0..MAX_ATTEMPTS {
        let mut stream = TcpStream::connect(addr)?;
        stream.set_read_timeout(Some(READ_TIMEOUT))?;
        stream.write_all(b"ORDER boots\n")?;
        let mut reply = [0u8; 2];
        match stream.read_exact(&mut reply) {
            Ok(()) => return Ok(String::from_utf8_lossy(&reply).into_owned()),
            Err(e) => last_err = Some(e),
        }
    }
    Err(last_err.unwrap())
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    let orders = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&orders);
    thread::spawn(move || {
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let counter = Arc::clone(&counter);
            thread::spawn(move || {
                let mut request = [0u8; 12];
                if stream.read_exact(&mut request).is_ok() {
                    if counter.fetch_add(1, Ordering::SeqCst) == 0 {
                        thread::sleep(SLOW_FIRST_REPLY);
                    }
                    let _ = stream.write_all(b"OK");
                }
            });
        }
    });
    let result = place_order(&addr);
    thread::sleep(SETTLE);
    println!("client result {:?}, orders created {}", result.is_ok(), orders.load(Ordering::SeqCst));
}
