package main

import (
	"fmt"
)

const startingCents = 1250
const formCents = "1425"

type Invoice struct{ cents any }

func (i *Invoice) apply(value string) { i.cents = value }
func main() {
	invoice := Invoice{startingCents}
	invoice.apply(formCents)
	fmt.Println("cents", invoice.cents.(int))
}
