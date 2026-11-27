package main

import "fmt"

func main() {
	city := "café"
	fmt.Printf("head %q\n", string([]rune(city)[:4]))
}
