package main

import "fmt"

func main() {
	ages := map[string]int{"jim": 30, "pam": 28}
	age, ok := ages["michael"]
	if !ok {
		fmt.Println("age unavailable")
		return
	}
	fmt.Println("age", age)
}
