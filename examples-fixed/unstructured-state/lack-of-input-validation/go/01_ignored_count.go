package main

import (
	"fmt"
	"strconv"
)

func main() {
	raw := "12abc"
	shipped, err := strconv.Atoi(raw)
	if err != nil {
		fmt.Println("token rejected")
		return
	}
	fmt.Println("shipped", shipped)
}
