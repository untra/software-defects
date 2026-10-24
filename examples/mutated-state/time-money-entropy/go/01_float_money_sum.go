package main

import "fmt"

const expectedTotal = 0.60

func main() {
	lineItems := []float64{0.10, 0.20, 0.30}
	var total float64
	for _, price := range lineItems {
		total += price
	}
	if total == expectedTotal {
		fmt.Println("ledger balanced")
	} else {
		fmt.Printf("ledger mismatch: %.17f\n", total)
	}
}
