fn billing_february(year: i32) -> i32 {
    if year % 400 == 0 || (year % 4 == 0 && year % 100 != 0) {
        29
    } else {
        28
    }
}

fn payroll_february(year: i32) -> i32 {
    billing_february(year)
}

fn main() {
    let year = 2024;
    println!("billing {year} february {}", billing_february(year));
    println!("payroll {year} february {}", payroll_february(year));
}
