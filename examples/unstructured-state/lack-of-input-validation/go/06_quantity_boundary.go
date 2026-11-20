package main

import "fmt"

const unitPrice = 10

func charge(quantity int) int {
	return quantity * unitPrice
}

func main() {
	fmt.Println("charge", charge(-3))
}
