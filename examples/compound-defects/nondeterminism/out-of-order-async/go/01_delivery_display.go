package main

import (
	"fmt"
)

var shown string

func load(id string, ready <-chan struct{}, done chan<- struct{}) {
	<-ready
	shown = id
	done <- struct{}{}
}
func main() {
	earlier, later, done := make(chan struct{}), make(chan struct{}), make(chan struct{})
	go load("orchard", earlier, done)
	go load("harbor", later, done)
	close(later)
	<-done
	close(earlier)
	<-done
	fmt.Println("shown", shown)
}
