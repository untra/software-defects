package main

import (
	"fmt"
	"strings"
)

func main() {
	line := "Host"
	parts := strings.SplitN(line, ":", 2)
	if len(parts) != 2 {
		fmt.Println("invalid header")
		return
	}
	fmt.Println("value", parts[1])
}
