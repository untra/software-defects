package main

import "fmt"

const centsPerUnit = 100

var currencySymbol = "$"

func formatPrice(cents int) string {
	return fmt.Sprintf("%s%d.%02d", currencySymbol, cents/centsPerUnit, cents%centsPerUnit)
}

func renderEuroInvoice(cents int) string {
	currencySymbol = "€"
	return "invoice: " + formatPrice(cents)
}

func main() {
	fmt.Println(formatPrice(1999))
	fmt.Println(renderEuroInvoice(2500))
	fmt.Println(formatPrice(1999))
}
