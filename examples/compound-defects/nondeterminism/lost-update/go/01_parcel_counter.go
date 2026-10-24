package main

import (
	"fmt"
	"sync"
)

const clerks = 2

var parcels int

func record(loaded chan<- struct{}, commit <-chan struct{}, done *sync.WaitGroup) {
	defer done.Done()
	value := parcels
	loaded <- struct{}{}
	<-commit
	value++
	parcels = value
}
func main() {
	loaded, commit := make(chan struct{}, clerks), make(chan struct{})
	var done sync.WaitGroup
	done.Add(clerks)
	for i := 0; i < clerks; i++ {
		go record(loaded, commit, &done)
	}
	for i := 0; i < clerks; i++ {
		<-loaded
	}
	close(commit)
	done.Wait()
	fmt.Println("parcels", parcels)
}
