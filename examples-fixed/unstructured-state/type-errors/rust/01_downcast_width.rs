use std::any::Any;

fn main() {
    let stored: Box<dyn Any> = Box::new(12_i64);
    let qty = stored
        .downcast_ref::<i64>()
        .copied()
        .expect("stored quantity must be i64");
    println!("qty {qty}");
}
