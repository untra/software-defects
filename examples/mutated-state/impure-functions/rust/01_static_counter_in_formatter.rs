use std::sync::atomic::{AtomicU32, Ordering};

static NEXT_INVOICE: AtomicU32 = AtomicU32::new(1);

fn format_invoice(customer: &str) -> String {
    let number = NEXT_INVOICE.fetch_add(1, Ordering::Relaxed);
    format!("INV-{:04}-{}", number, customer)
}

fn main() {
    let preview = format_invoice("acme");
    let issued = format_invoice("acme");
    println!("preview {} / issued {}", preview, issued);
}
