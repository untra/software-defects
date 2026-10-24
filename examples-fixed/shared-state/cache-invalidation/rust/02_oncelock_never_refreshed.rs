use std::sync::Mutex;

static FLAG_SOURCE: Mutex<bool> = Mutex::new(false);

fn in_maintenance() -> bool {
    *FLAG_SOURCE.lock().unwrap()
}

fn set_maintenance(on: bool) {
    *FLAG_SOURCE.lock().unwrap() = on;
}

fn main() {
    println!("maintenance before: {}", in_maintenance());
    set_maintenance(true);
    println!(
        "maintenance after operator enabled it: {}",
        in_maintenance()
    );
}
