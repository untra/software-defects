use std::any::Any;
struct Settings {
    retries: usize,
}
fn retry_count(value: &dyn Any) -> Option<usize> {
    let settings = value.downcast_ref::<Settings>()?;
    Some(settings.retries)
}
fn main() {
    println!("retries {:?}", retry_count(&"configuration unavailable"));
}
