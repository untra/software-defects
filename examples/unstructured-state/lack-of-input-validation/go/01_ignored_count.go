package main

import (
	"fmt"
	"strconv"
)

func main() {
	raw := "12abc"
	shipped, _ := strconv.Atoi(raw)
	fmt.Println("shipped", shipped)
}
