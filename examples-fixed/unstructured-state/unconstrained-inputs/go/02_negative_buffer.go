package main

import "fmt"

const maxBufferBytes = 1024

func main() {
	requested := -1
	if requested < 0 || requested > maxBufferBytes {
		fmt.Println("request rejected")
		return
	}
	buf := make([]byte, requested)
	fmt.Println("bytes", len(buf))
}
