fn billing_february(year: i32) -> i32 {
    if year % 4 == 0 {
        29
    } else {
        28
    }
}

fn payroll_february(_year: i32) -> i32 {
    28
}

fn main() {
    let year = 2024;
    println!("billing {year} february {}", billing_february(year));
    println!("payroll {year} february {}", payroll_february(year));
}
