const MAX_BATCH_ITEMS: usize = 3;

fn import_rows(rows: &[&str]) -> Result<usize, &'static str> {
    if rows.len() > MAX_BATCH_ITEMS {
        return Err("batch rejected");
    }
    let mut processed = 0;
    for _ in rows {
        processed += 1;
    }
    Ok(processed)
}

fn main() {
    match import_rows(&["a", "b", "c", "d"]) {
        Ok(processed) => println!("processed {processed}"),
        Err(error) => println!("{error}"),
    }
}
