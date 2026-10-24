package main

import "fmt"

func main() {
	requested := -1
	buf := make([]byte, requested)
	fmt.Println("bytes", len(buf))
}
