fn main() {
    let raw = "12abc";
    let Ok(shipped) = raw.parse::<i32>() else {
        println!("token rejected");
        return;
    };
    println!("shipped {shipped}");
}
