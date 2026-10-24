package main

import (
	"fmt"
)

const initialStatus = "packing"
const nextStatus = "shipped"

var primary = initialStatus
var replica = initialStatus

func copyStatus(ready <-chan struct{}, done chan<- struct{}) { <-ready; replica = primary; close(done) }
func main() {
	ready, done := make(chan struct{}), make(chan struct{})
	primary = nextStatus
	go copyStatus(ready, done)
	fmt.Println("status", replica)
	close(ready)
	<-done
}
