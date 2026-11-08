fn main() {
    let city = "café";
    let head: String = city.chars().take(4).collect();
    println!("{head}");
}
