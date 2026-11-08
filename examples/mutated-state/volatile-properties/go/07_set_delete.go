package main

import (
	"fmt"
	"maps"
)

func main() {
	source := map[string]struct{}{"a": {}, "b": {}}
	work := source
	delete(work, "a")
	if !(len(work) == 1) {
		panic("wrong result")
	}
	if !(maps.Equal(source, map[string]struct{}{"a": {}, "b": {}})) {
		panic("source changed")
	}
	fmt.Println("ok")
}
