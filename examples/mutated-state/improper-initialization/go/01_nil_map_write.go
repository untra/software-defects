package main

import "fmt"

type Inventory struct {
	counts map[string]int
}

func (i *Inventory) Add(sku string) {
	i.counts[sku]++
}

func main() {
	var inv Inventory
	inv.Add("sku-1")
	fmt.Println(inv.counts)
}
