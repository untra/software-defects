package main

import (
	"fmt"
)

const initialQuantity = 2

var cart = map[string]*int{}

func apply(payload map[string]*int) { cart["qty"] = payload["qty"] }
func main() {
	qty := initialQuantity
	cart["qty"] = &qty
	apply(map[string]*int{})
	fmt.Println("quantity", *cart["qty"])
}
