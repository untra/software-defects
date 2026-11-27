package main

import (
	"fmt"
	"maps"
)

func main() {
	source := map[string]int{"a": 1, "b": 2}
	work := maps.Clone(source)
	work["c"] = 9
	if !(len(work) == 3) {
		panic("wrong result")
	}
	if !(maps.Equal(source, map[string]int{"a": 1, "b": 2})) {
		panic("source changed")
	}
	fmt.Println("ok")
}
