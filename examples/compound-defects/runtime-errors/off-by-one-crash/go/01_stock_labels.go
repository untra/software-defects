package main

import (
	"fmt"
)

type Item struct{ sku string }

var items = []Item{{"pen"}, {"pad"}}

func collect(items []Item) []string {
	labels := []string{}
	i := 0
	for i <= len(items) {
		labels = append(labels, items[i].sku)
		i++
	}
	return labels
}
func main() { fmt.Println("labels", collect(items)) }
