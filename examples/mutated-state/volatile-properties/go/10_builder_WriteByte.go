package main

import (
	"fmt"
	"strings"
)

func main() {
	source := strings.Builder{}
	source.WriteString("abc")
	work := &source
	err := work.WriteByte('Z')
	if !(err == nil && work.String() == "abcZ") {
		panic("wrong result")
	}
	if !(source.String() == "abc") {
		panic("source changed")
	}
	fmt.Println("ok")
}
