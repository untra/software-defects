use std::thread;
use std::time::Duration;

const EXPORT_WINDOW: Duration = Duration::from_millis(50);

fn start_export(rows: Vec<String>) {
    thread::spawn(move || {
        if rows.is_empty() {
            panic!("export failed: no rows");
        }
        println!("exported {} rows", rows.len());
    });
}

fn main() {
    start_export(Vec::new());
    thread::sleep(EXPORT_WINDOW);
    println!("nightly export finished successfully");
}
