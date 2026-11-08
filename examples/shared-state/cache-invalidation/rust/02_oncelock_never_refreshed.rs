use std::sync::{Mutex, OnceLock};

static FLAG_SOURCE: Mutex<bool> = Mutex::new(false);
static MAINTENANCE_MODE: OnceLock<bool> = OnceLock::new();

fn in_maintenance() -> bool {
    *MAINTENANCE_MODE.get_or_init(|| *FLAG_SOURCE.lock().unwrap())
}

fn set_maintenance(on: bool) {
    *FLAG_SOURCE.lock().unwrap() = on;
}

fn main() {
    println!("maintenance before: {}", in_maintenance());
    set_maintenance(true);
    println!("maintenance after operator enabled it: {}", in_maintenance());
}
