package main

import "fmt"

func average(scores []int) (int, bool) {
	if len(scores) == 0 {
		return 0, false
	}
	total := 0
	for _, score := range scores {
		total += score
	}
	return total / len(scores), true
}

func main() {
	value, ok := average(nil)
	fmt.Println("average", value, "available", ok)
}
