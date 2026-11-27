package main

import "fmt"

func main() {
	status := "BLOCKED"
	if status == "blocked" {
		fmt.Println("denied")
	} else {
		fmt.Println("allowed")
	}
}
