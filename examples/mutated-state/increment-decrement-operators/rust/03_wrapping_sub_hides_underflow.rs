fn withdraw(balance: u32, amount: u32) -> u32 {
    balance.wrapping_sub(amount)
}

fn main() {
    let remaining = withdraw(50, 80);
    println!("remaining balance: {}", remaining);
}
