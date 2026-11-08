package main

import (
	"fmt"
)

const requestedSKU = "pad"
const itemCents = 150

type Item struct {
	sku   string
	cents int
}

var catalog = []Item{{"pen", itemCents}}

func lookup(sku string) *Item {
	for i := range catalog {
		if catalog[i].sku == sku {
			return &catalog[i]
		}
	}
	return nil
}
func main() { selected := lookup(requestedSKU); fmt.Println("cents", selected.cents) }
