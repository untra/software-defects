package main

import (
	"fmt"
)

const capacity = 1
const startingTick = 100
const finishingTick = 105

var tick = startingTick
var occupied int

func report(ready <-chan struct{}, acquired chan<- struct{}, done chan<- struct{}) {
	occupied++
	started := tick
	close(acquired)
	<-ready
	fmt.Println("held ticks", tick-started)
	occupied--
	close(done)
}
func main() {
	ready, acquired, done := make(chan struct{}), make(chan struct{}), make(chan struct{})
	go report(ready, acquired, done)
	<-acquired
	tick = finishingTick
	fmt.Println("pool busy", occupied == capacity)
	close(ready)
	<-done
}
