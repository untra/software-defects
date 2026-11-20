const MAX_BUFFER_BYTES: usize = 8;

fn allocate(requested: usize) -> Result<Vec<u8>, &'static str> {
    Ok(vec![0; requested])
}

fn main() {
    match allocate(MAX_BUFFER_BYTES + 1) {
        Ok(buffer) => println!("bytes {}", buffer.len()),
        Err(error) => println!("{error}"),
    }
}
