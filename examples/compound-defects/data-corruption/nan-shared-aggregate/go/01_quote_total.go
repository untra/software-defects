package main

import (
	"fmt"
	"sync"
)

const openingTotal = 100.0

var total = openingTotal
var ledger sync.Mutex

func postQuote(amount, units float64) {
	ledger.Lock()
	defer ledger.Unlock()
	quote := amount / units
	total += quote
}
func main() { postQuote(0, 0); fmt.Println("total", total) }
