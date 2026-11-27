package main

import "fmt"

const centsPerUnit = 100

const currencySymbol = "$"

func formatPrice(cents int) string { return priceWithSymbol(cents, currencySymbol) }

func priceWithSymbol(cents int, symbol string) string {
	return fmt.Sprintf("%s%d.%02d", symbol, cents/centsPerUnit, cents%centsPerUnit)
}

func renderEuroInvoice(cents int) string {
	return "invoice: " + priceWithSymbol(cents, "€")
}

func main() {
	fmt.Println(formatPrice(1999))
	fmt.Println(renderEuroInvoice(2500))
	fmt.Println(formatPrice(1999))
}
