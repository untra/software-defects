use std::fs;
const RECORD_BYTES: usize = 4;
const REPLACEMENT: &[u8] = b"PAID";
#[repr(C)]
struct Page { header: [u8; RECORD_BYTES], records: [u8; RECORD_BYTES] }
fn find(records: &[u8], key: &[u8]) -> isize {
    records.windows(key.len()).position(|record| record == key).map(|index| index as isize).unwrap_or(-1)
}
fn main() {
    let path = std::env::temp_dir().join(format!("accounts-{}.dat", std::process::id()));
    fs::write(&path, b"OPENpear").unwrap();
    let bytes = fs::read(&path).unwrap();
    let mut page = Page { header: bytes[..RECORD_BYTES].try_into().unwrap(), records: bytes[RECORD_BYTES..].try_into().unwrap() };
    let index = find(&page.records, b"plum");
    unsafe {
        let destination = page.records.as_mut_ptr().offset(index * RECORD_BYTES as isize);
        std::ptr::copy_nonoverlapping(REPLACEMENT.as_ptr(), destination, REPLACEMENT.len());
    }
    let output = [&page.header[..], &page.records[..]].concat();
    fs::write(&path, output).unwrap();
    println!("header {}", String::from_utf8_lossy(&fs::read(&path).unwrap()[..RECORD_BYTES]));
    fs::remove_file(path).unwrap();
}
