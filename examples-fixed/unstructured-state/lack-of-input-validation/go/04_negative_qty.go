package main

import "fmt"

func main() {
	qty := -3
	price := 10
	if qty < 0 {
		fmt.Println("quantity rejected")
		return
	}
	fmt.Println("charge", qty*price)
}
