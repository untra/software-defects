package main

import (
	"fmt"
)

var slots = []int{1, 2, 3}
var replacement = []int{10, 20, 30}

func scan(scanned chan<- struct{}, changed <-chan struct{}, result chan<- int) {
	total := 0
	for i := 0; i < len(slots); i++ {
		total += slots[i]
		if i == 0 {
			close(scanned)
			<-changed
		}
	}
	result <- total
}
func main() {
	scanned, changed, result := make(chan struct{}), make(chan struct{}), make(chan int)
	go scan(scanned, changed, result)
	go func() { <-scanned; copy(slots, replacement); close(changed) }()
	fmt.Println("units", <-result)
}
