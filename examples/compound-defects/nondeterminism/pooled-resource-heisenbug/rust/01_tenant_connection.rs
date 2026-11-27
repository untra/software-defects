use std::cell::RefCell;
thread_local! { static POOL: RefCell<Vec<String>> = RefCell::new(vec![String::new()]); }
fn borrow() -> String { POOL.with(|pool| pool.borrow_mut().pop().unwrap()) }
fn release(connection: String) { POOL.with(|pool| pool.borrow_mut().push(connection)); }
fn select_tenant(connection: &mut String, tenant: &str) { *connection = tenant.to_owned(); }
fn main() {
    let mut connection = borrow();
    select_tenant(&mut connection, "orchard");
    release(connection);
    let next = borrow();
    println!("tenant {next}");
    release(next);
}
