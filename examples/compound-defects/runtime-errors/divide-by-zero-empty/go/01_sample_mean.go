package main

import (
	"fmt"
)

func mean(samples []int) int {
	total := 0
	for _, sample := range samples {
		total += sample
	}
	return total / len(samples)
}
func main() { samples := []int{}; fmt.Println("mean", mean(samples)) }
