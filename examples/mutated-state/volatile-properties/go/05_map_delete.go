package main

import (
	"fmt"
	"maps"
)

func main() {
	source := map[string]int{"a": 1, "b": 2}
	work := source
	delete(work, "a")
	if !(len(work) == 1) {
		panic("wrong result")
	}
	if !(maps.Equal(source, map[string]int{"a": 1, "b": 2})) {
		panic("source changed")
	}
	fmt.Println("ok")
}
