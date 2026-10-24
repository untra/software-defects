package main

import (
	"fmt"
	"strings"
)

const maxLabelLength = 6

func main() {
	requested := 8
	if requested < 0 || requested > maxLabelLength {
		fmt.Println("label rejected")
		return
	}
	label := strings.Repeat("x", requested)
	fmt.Println("label", label)
}
