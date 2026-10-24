package main

import "fmt"

type Address struct {
	City string
}

type Customer struct {
	Name     string
	Shipping *Address
}

func shippingLabel(c Customer) string {
	if c.Shipping == nil {
		return "shipping address required"
	}
	return c.Name + ", " + c.Shipping.City
}

func main() {
	c := Customer{Name: "ana"}
	fmt.Println(shippingLabel(c))
}
