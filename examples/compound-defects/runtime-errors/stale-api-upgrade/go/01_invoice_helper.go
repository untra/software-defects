package main

import (
	"fmt"
)

const invoiceCents = 450

type Billing map[string]func() int
type ReceiptHelper struct{ billing Billing }

func (h ReceiptHelper) receipt() int { return h.billing["amount"]() }

type Invoice struct{ ReceiptHelper }

func main() {
	billing := Billing{"totalCents": func() int { return invoiceCents }}
	invoice := Invoice{ReceiptHelper{billing}}
	fmt.Println("receipt", invoice.receipt())
}
