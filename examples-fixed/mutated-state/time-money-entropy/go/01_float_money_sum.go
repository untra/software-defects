package main

import "fmt"

const expectedTotal = 60

func main() {
	lineItems := []int64{10, 20, 30}
	var total int64
	for _, price := range lineItems {
		total += price
	}
	if total == expectedTotal {
		fmt.Println("ledger balanced")
	} else {
		fmt.Printf("ledger mismatch: %d\n", total)
	}
}
