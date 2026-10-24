package main

import "fmt"

func main() {
	var fileSize int64 = 3_000_000_000
	contentLength := fileSize
	fmt.Println("content-length:", contentLength)
}
