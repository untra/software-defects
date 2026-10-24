package main

import (
	"fmt"
	"sync"
)

const millis int64 = 1700000000000
const callers = 2

var receipts = map[string]string{}
var mu sync.Mutex

func issue(customer string, checked chan<- struct{}, commit <-chan struct{}, done chan<- struct{}) {
	id := fmt.Sprintf("receipt-%d", millis)
	mu.Lock()
	_, exists := receipts[id]
	mu.Unlock()
	checked <- struct{}{}
	<-commit
	if !exists {
		mu.Lock()
		receipts[id] = customer
		mu.Unlock()
	}
	done <- struct{}{}
}
func main() {
	checked, commit, done := make(chan struct{}, callers), make(chan struct{}), make(chan struct{}, callers)
	go issue("orchard", checked, commit, done)
	go issue("harbor", checked, commit, done)
	for i := 0; i < callers; i++ {
		<-checked
	}
	close(commit)
	for i := 0; i < callers; i++ {
		<-done
	}
	fmt.Println("receipts", len(receipts))
}
