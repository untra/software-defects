package main

import "fmt"

const unitPrice = 10

func charge(quantity int) (int, error) {
	if quantity < 0 {
		return 0, fmt.Errorf("quantity rejected")
	}
	return quantity * unitPrice, nil
}

func main() {
	total, err := charge(-3)
	if err != nil {
		fmt.Println(err)
		return
	}
	fmt.Println("charge", total)
}
