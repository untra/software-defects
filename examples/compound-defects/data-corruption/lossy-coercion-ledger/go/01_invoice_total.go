package main

import (
	"fmt"
	"sync"
)

const invoiceAmount = 19.75

var total int
var ledger sync.Mutex

func postInvoice(amount float64) {
	ledger.Lock()
	defer ledger.Unlock()
	whole := int(amount)
	total += whole
}
func main() { postInvoice(invoiceAmount); fmt.Println("total", total) }
