package main

import "fmt"

func main() {
	unitPrice := int64(3_000_000_000_000_000_000)
	units := int64(4)
	total := unitPrice * units
	fmt.Println("total", total)
}
