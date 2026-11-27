package main

import "fmt"

func billingFebruary(year int) int {
	if year%400 == 0 || (year%4 == 0 && year%100 != 0) {
		return 29
	}
	return 28
}

func payrollFebruary(year int) int {
	return billingFebruary(year)
}

func main() {
	year := 2024
	fmt.Printf("billing %d february %d\n", year, billingFebruary(year))
	fmt.Printf("payroll %d february %d\n", year, payrollFebruary(year))
}
