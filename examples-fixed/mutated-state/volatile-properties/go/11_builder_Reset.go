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
	work.Reset()
	if !(work.String() == "") {
		panic("wrong result")
	}
	if !(source.String() == "abc") {
		panic("source changed")
	}
	fmt.Println("ok")
}
