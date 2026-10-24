package main

import (
	"fmt"
	"strings"
)

func main() {
	requested := 8
	label := strings.Repeat("x", requested)
	fmt.Println("label", label)
}
