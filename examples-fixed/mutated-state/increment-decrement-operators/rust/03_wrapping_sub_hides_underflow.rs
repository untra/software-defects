fn withdraw(balance: u32, amount: u32) -> Option<u32> {
    balance.checked_sub(amount)
}

fn main() {
    let remaining = withdraw(50, 80);
    println!("remaining balance: {:?}", remaining);
}
