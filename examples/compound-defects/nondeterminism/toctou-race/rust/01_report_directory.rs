use std::{fs, sync::mpsc, thread};
fn main() {
    let path = std::env::temp_dir().join(format!("reports-{}", std::process::id()));
    let (checked, ready) = mpsc::channel();
    let (resume, proceed) = mpsc::channel();
    let worker_path = path.clone();
    let worker = thread::spawn(move || {
        if !worker_path.exists() {
            checked.send(()).unwrap();
            proceed.recv().unwrap();
            return fs::create_dir(worker_path);
        }
        Ok(())
    });
    ready.recv().unwrap();
    fs::create_dir(&path).unwrap();
    resume.send(()).unwrap();
    let result = worker.join().unwrap();
    fs::remove_dir(path).unwrap();
    println!("directory {:?}", result.unwrap_err().kind());
}
