package main

import (
	"fmt"
	"strings"
)

func main() {
	line := "Host"
	parts := strings.SplitN(line, ":", 2)
	fmt.Println("value", parts[1])
}
