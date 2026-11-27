package main

import (
	"fmt"
)

var guests = []string{"Ari", "Bo", "Cy", "Dee"}

func roster(names []string) int {
	accepted := []string{}
	comparisons := 0
	for _, name := range names {
		present := false
		for _, earlier := range accepted {
			comparisons++
			if earlier == name {
				present = true
			}
		}
		if !present {
			accepted = append(accepted, name)
		}
	}
	return comparisons
}
func main() { fmt.Println("comparisons", roster(guests)) }
