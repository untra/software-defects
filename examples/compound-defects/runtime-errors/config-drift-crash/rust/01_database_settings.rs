use std::env;
use std::panic;
const ADDRESS: &str = "postgres://warehouse";
const PREFIX_LENGTH: usize = 8;
fn configure() { env::set_var("DB_URL", ADDRESS); }
fn prefix() -> String { env::var("DATABASE_URL").unwrap()[..PREFIX_LENGTH].to_owned() }
fn main() {
    let old_writer = env::var_os("DB_URL");
    let old_reader = env::var_os("DATABASE_URL");
    let result = panic::catch_unwind(|| {
        env::remove_var("DATABASE_URL");
        configure();
        println!("database {}", prefix());
    });
    for (key, value) in [("DB_URL", old_writer), ("DATABASE_URL", old_reader)] {
        match value { Some(value) => env::set_var(key, value), None => env::remove_var(key) }
    }
    if let Err(error) = result { panic::resume_unwind(error); }
}
