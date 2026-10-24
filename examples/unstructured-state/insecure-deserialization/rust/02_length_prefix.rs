fn read_record(buf: &[u8]) -> (String, u8) {
    let name_len = buf[0] as usize;
    let name = String::from_utf8_lossy(&buf[1..1 + name_len]).into_owned();
    let flag = buf[1 + name_len];
    (name, flag)
}

fn main() {
    let buf = [3_u8, b'a', b'b', 1, 9];
    let (name, flag) = read_record(&buf);
    println!("name={name:?} flag={flag}");
}
