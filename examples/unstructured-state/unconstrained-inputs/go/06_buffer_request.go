package main

import "fmt"

const maxBufferBytes = 8

func allocate(requested int) ([]byte, error) {
	return make([]byte, requested), nil
}

func main() {
	buffer, err := allocate(maxBufferBytes + 1)
	if err != nil {
		fmt.Println(err)
		return
	}
	fmt.Println("bytes", len(buffer))
}
