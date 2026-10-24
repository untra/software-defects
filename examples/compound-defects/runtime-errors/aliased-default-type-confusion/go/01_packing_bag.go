package main

import (
	"fmt"
)

var defaultBag = []any{}

const middleNote = "ink"

func stash(note any, bag *[]any) *[]any {
	if bag == nil {
		bag = &defaultBag
	}
	*bag = append(*bag, note)
	return bag
}
func main() {
	stash(map[string]string{"sku": "pen"}, nil)
	stash(middleNote, nil)
	bag := stash(map[string]string{"sku": "pad"}, nil)
	labels := []string{}
	for _, note := range *bag {
		labels = append(labels, note.(map[string]string)["sku"])
	}
	fmt.Println("labels", labels)
}
