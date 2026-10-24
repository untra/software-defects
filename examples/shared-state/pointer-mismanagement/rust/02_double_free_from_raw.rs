struct Session {
    token: String,
}

fn into_handle(session: Session) -> *mut Session {
    Box::into_raw(Box::new(session))
}

fn close(handle: *mut Session) {
    let session = unsafe { Box::from_raw(handle) };
    println!("closing session {}", session.token);
}

fn main() {
    let handle = into_handle(Session { token: "abc123".to_string() });
    close(handle);
    close(handle);
}
