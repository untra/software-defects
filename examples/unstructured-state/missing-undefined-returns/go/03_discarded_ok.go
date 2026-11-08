package main

import "fmt"

func main() {
	rates := map[string]int{"usd": 100}
	cents, _ := rates["eur"]
	fmt.Printf("eur %d\n", cents)
}
