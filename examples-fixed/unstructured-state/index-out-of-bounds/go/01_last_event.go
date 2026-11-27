package main

import "fmt"

func main() {
	var events []string
	if len(events) == 0 {
		fmt.Println("no events")
		return
	}
	fmt.Println("last", events[len(events)-1])
}
