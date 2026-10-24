use std::io::{self, Write};

const MAX_SEGMENT: usize = 4;

struct Socket {
    received: Vec<u8>,
}

impl Write for Socket {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = buf.len().min(MAX_SEGMENT);
        self.received.extend_from_slice(&buf[..n]);
        Ok(n)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn send(socket: &mut impl Write, message: &[u8]) -> io::Result<()> {
    socket.write_all(message)?;
    Ok(())
}

fn main() {
    let mut socket = Socket {
        received: Vec::new(),
    };
    let message = b"{\"amount\":50}";
    send(&mut socket, message).unwrap();
    println!(
        "peer received {:?} ({} of {} bytes)",
        String::from_utf8_lossy(&socket.received),
        socket.received.len(),
        message.len()
    );
}
