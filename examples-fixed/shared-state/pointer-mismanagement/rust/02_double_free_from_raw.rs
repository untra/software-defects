struct Session {
    token: String,
}

fn into_handle(session: Session) -> Option<Box<Session>> {
    Some(Box::new(session))
}

fn close(handle: &mut Option<Box<Session>>) {
    let Some(session) = handle.take() else {
        return;
    };
    println!("closing session {}", session.token);
}

fn main() {
    let mut handle = into_handle(Session {
        token: "abc123".to_string(),
    });
    close(&mut handle);
    close(&mut handle);
}
