package main

import (
	"fmt"
	"strings"
)

const allowedStatus = "active"

func main() {
	status := "BLOCKED"
	if strings.EqualFold(status, allowedStatus) {
		fmt.Println("allowed")
	} else {
		fmt.Println("denied")
	}
}
