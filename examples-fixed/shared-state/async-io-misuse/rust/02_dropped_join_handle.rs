use std::thread;
use std::time::Duration;

const EXPORT_WINDOW: Duration = Duration::from_millis(50);

fn start_export(rows: Vec<String>) -> Result<(), String> {
    thread::spawn(move || {
        if rows.is_empty() {
            return Err("export failed: no rows".to_string());
        }
        println!("exported {} rows", rows.len());
        Ok(())
    })
    .join()
    .map_err(|_| "export worker panicked".to_string())?
}

fn main() {
    if let Err(error) = start_export(Vec::new()) {
        println!("{error}");
        return;
    }
    thread::sleep(EXPORT_WINDOW);
    println!("nightly export finished successfully");
}
