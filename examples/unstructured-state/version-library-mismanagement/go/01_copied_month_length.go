package main

import "fmt"

func billingFebruary(year int) int {
	if year%4 == 0 {
		return 29
	}
	return 28
}

func payrollFebruary(_ int) int {
	return 28
}

func main() {
	year := 2024
	fmt.Printf("billing %d february %d\n", year, billingFebruary(year))
	fmt.Printf("payroll %d february %d\n", year, payrollFebruary(year))
}
