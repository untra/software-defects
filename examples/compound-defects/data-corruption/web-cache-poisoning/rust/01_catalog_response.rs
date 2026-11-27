use std::collections::HashMap;
const PATH: &str = "/catalog";
const DEFAULT_HEADING: &str = "Public catalog";
fn render(cache: &mut HashMap<String, String>, path: &str, query: &HashMap<&str, &str>) -> String {
    if let Some(body) = cache.get(path) { return body.clone(); }
    let body = query.get("heading").copied().unwrap_or(DEFAULT_HEADING).to_owned();
    cache.insert(path.to_owned(), body.clone());
    body
}
fn main() {
    let mut cache = HashMap::new();
    render(&mut cache, PATH, &HashMap::from([("heading", "Members only")]));
    println!("public page {}", render(&mut cache, PATH, &HashMap::new()));
}
