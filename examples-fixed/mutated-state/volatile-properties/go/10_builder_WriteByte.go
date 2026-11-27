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
	err := work.WriteByte('Z')
	if !(err == nil && work.String() == "abcZ") {
		panic("wrong result")
	}
	if !(source.String() == "abc") {
		panic("source changed")
	}
	fmt.Println("ok")
}
