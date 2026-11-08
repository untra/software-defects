package main

import (
	"fmt"
)

type Context struct{ user string }

var context Context

func (c *Context) show(user string, ready <-chan struct{}, assigned chan<- struct{}, result chan<- string) {
	c.user = user
	close(assigned)
	<-ready
	result <- c.user
}
func main() {
	ready, assigned, result := make(chan struct{}), make(chan struct{}), make(chan string)
	go context.show("orchard", ready, assigned, result)
	<-assigned
	secondReady, secondAssigned, secondResult := make(chan struct{}), make(chan struct{}), make(chan string)
	go context.show("harbor", secondReady, secondAssigned, secondResult)
	<-secondAssigned
	close(secondReady)
	<-secondResult
	close(ready)
	fmt.Println("orchard received", <-result)
}
