use std::sync::Mutex;
static LOCALE: Mutex<String> = Mutex::new(String::new());
fn render(locale: Option<&str>) -> String {
    if let Some(locale) = locale { *LOCALE.lock().unwrap() = locale.to_owned(); }
    LOCALE.lock().unwrap().clone()
}
fn main() {
    render(Some("staff-preview"));
    println!("next locale {}", render(None));
}
