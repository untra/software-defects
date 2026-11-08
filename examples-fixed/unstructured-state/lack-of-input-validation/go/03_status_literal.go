package main

import (
	"fmt"
	"strings"
)

func main() {
	status := "BLOCKED"
	if strings.EqualFold(status, "blocked") {
		fmt.Println("denied")
	} else {
		fmt.Println("allowed")
	}
}
