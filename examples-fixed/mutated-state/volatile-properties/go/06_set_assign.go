package main

import (
	"fmt"
	"maps"
)

func main() {
	source := map[string]struct{}{"a": {}, "b": {}}
	work := maps.Clone(source)
	work["c"] = struct{}{}
	if !(len(work) == 3) {
		panic("wrong result")
	}
	if !(maps.Equal(source, map[string]struct{}{"a": {}, "b": {}})) {
		panic("source changed")
	}
	fmt.Println("ok")
}
