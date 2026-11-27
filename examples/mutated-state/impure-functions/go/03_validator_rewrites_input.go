package main

import (
	"fmt"
	"strings"
)

const minQuantity = 1

type Order struct {
	Email    string
	Quantity int
}

func isValid(o *Order) bool {
	o.Email = strings.ToLower(strings.TrimSpace(o.Email))
	if o.Quantity < minQuantity {
		o.Quantity = minQuantity
	}
	return o.Email != ""
}

func main() {
	order := &Order{Email: " Ana@Example.com ", Quantity: 0}
	fmt.Println("valid:", isValid(order), "quantity to ship:", order.Quantity)
}
