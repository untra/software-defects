package main

import (
	"fmt"
	"strings"
)

func main() {
	source := strings.Builder{}
	source.WriteString("abc")
	work := &source
	work.Reset()
	if !(work.String() == "") {
		panic("wrong result")
	}
	if !(source.String() == "abc") {
		panic("source changed")
	}
	fmt.Println("ok")
}
