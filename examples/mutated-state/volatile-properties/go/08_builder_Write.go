package main

import (
	"fmt"
	"strings"
)

func main() {
	source := strings.Builder{}
	source.WriteString("abc")
	work := &source
	count, err := work.Write([]byte("Z"))
	if !(count == 1 && err == nil && work.String() == "abcZ") {
		panic("wrong result")
	}
	if !(source.String() == "abc") {
		panic("source changed")
	}
	fmt.Println("ok")
}
