use std::{fs, sync::mpsc, thread};
const STAMP: u64 = 1700000000000;
fn main() {
    let directory = std::env::temp_dir().join(format!("invoices-{}", std::process::id()));
    fs::create_dir(&directory).unwrap();
    let path = directory.join(format!("invoice-{STAMP}"));
    let (checked, ready) = mpsc::channel();
    let (resume, proceed) = mpsc::channel();
    let destination = path.clone();
    let writer = thread::spawn(move || {
        if !destination.exists() {
            checked.send(()).unwrap();
            proceed.recv().unwrap();
            fs::write(destination, "orchard").unwrap();
        }
    });
    ready.recv().unwrap();
    fs::write(&path, "harbor").unwrap();
    resume.send(()).unwrap();
    writer.join().unwrap();
    println!("invoice {}", fs::read_to_string(path).unwrap());
    fs::remove_dir_all(directory).unwrap();
}
