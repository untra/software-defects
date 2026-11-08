package main

import "fmt"

func average(scores []int) int {
	total := 0
	for _, score := range scores {
		total += score
	}
	return total / len(scores)
}

func main() {
	fmt.Println("average", average(nil))
}
