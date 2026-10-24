package main

import "fmt"

func main() {
	rates := map[string]int{"usd": 100}
	cents, ok := rates["eur"]
	if !ok {
		fmt.Println("rate unavailable")
		return
	}
	fmt.Printf("eur %d\n", cents)
}
