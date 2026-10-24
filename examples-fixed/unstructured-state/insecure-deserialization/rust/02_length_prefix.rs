fn read_record(buf: &[u8]) -> Option<(String, u8)> {
    let name_len = *buf.first()? as usize;
    let name = std::str::from_utf8(buf.get(1..1 + name_len)?).ok()?;
    if name.chars().any(char::is_control) {
        return None;
    }
    let flag = *buf.get(1 + name_len)?;
    Some((name.to_owned(), flag))
}

fn main() {
    let buf = [3_u8, b'a', b'b', 1, 9];
    let Some((name, flag)) = read_record(&buf) else {
        println!("invalid record");
        return;
    };
    println!("name={name:?} flag={flag}");
}
