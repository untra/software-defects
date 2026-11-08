package main

import (
	"fmt"
	"strings"
)

func main() {
	source := strings.Builder{}
	source.WriteString("abc")
	work := &strings.Builder{}
	work.WriteString(source.String())
	count, err := work.WriteString("Z")
	if !(count == 1 && err == nil && work.String() == "abcZ") {
		panic("wrong result")
	}
	if !(source.String() == "abc") {
		panic("source changed")
	}
	fmt.Println("ok")
}
