package main

import "fmt"

func main() {
	voltage := 12
	resistance := 0
	if resistance == 0 {
		fmt.Println("zero divisor")
		return
	}
	current := voltage / resistance
	fmt.Println("current", current)
}
