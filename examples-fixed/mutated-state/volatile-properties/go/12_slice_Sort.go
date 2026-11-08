package main

import (
	"fmt"
	"slices"
)

func main() {
	source := []int{3, 1, 2}
	work := slices.Clone(source)
	slices.Sort(work)
	if !(slices.Equal(work, []int{1, 2, 3})) {
		panic("wrong result")
	}
	if !(slices.Equal(source, []int{3, 1, 2})) {
		panic("source changed")
	}
	fmt.Println("ok")
}
