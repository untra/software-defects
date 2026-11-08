package main

import (
	"fmt"
)

const largestCount = 255
const requestedCount = 256

func main() {
	count := uint8(largestCount)
	count++
	slots := make([]byte, int(count))
	for i := 0; i < requestedCount; i++ {
		slots[i] = 'P'
	}
	fmt.Println("slots", len(slots))
}
