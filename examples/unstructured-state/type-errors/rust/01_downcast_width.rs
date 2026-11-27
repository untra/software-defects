use std::any::Any;

fn main() {
    let stored: Box<dyn Any> = Box::new(12_i64);
    let qty = stored.downcast_ref::<i32>().copied().unwrap_or(0);
    println!("qty {qty}");
}
