package main

import (
	"fmt"
)

const openingBalance uint64 = 100
const postingCapacity = 1
const refund int32 = -1

var balance = openingBalance
var slots = make(chan struct{}, postingCapacity)

func post(value int32, held chan<- struct{}, ready <-chan struct{}, done chan<- struct{}) {
	slots <- struct{}{}
	close(held)
	<-ready
	balance += uint64(uint32(value))
	<-slots
	close(done)
}
func main() {
	held, ready, done := make(chan struct{}), make(chan struct{}), make(chan struct{})
	go post(refund, held, ready, done)
	<-held
	select {
	case slots <- struct{}{}:
		fmt.Println("pool available")
		<-slots
	default:
		fmt.Println("pool busy")
	}
	close(ready)
	<-done
	fmt.Println("balance", balance)
}
