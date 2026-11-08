use std::sync::atomic::{AtomicU32, Ordering};

static NEXT_INVOICE: AtomicU32 = AtomicU32::new(1);

fn format_invoice(customer: &str, number: u32) -> String {
    format!("INV-{:04}-{}", number, customer)
}

fn main() {
    let number = NEXT_INVOICE.fetch_add(1, Ordering::Relaxed);
    let preview = format_invoice("acme", number);
    let issued = format_invoice("acme", number);
    println!("preview {} / issued {}", preview, issued);
}
