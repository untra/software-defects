package main

import (
	"fmt"
)

type Plugin struct{ routes map[string]func() string }

var routes = map[string]func() string{}

const route = "shipping"

func install(module Plugin, ready <-chan struct{}, done chan<- struct{}) {
	<-ready
	for name, handler := range module.routes {
		routes[name] = handler
	}
	done <- struct{}{}
}
func main() {
	standard, express, done := make(chan struct{}), make(chan struct{}), make(chan struct{})
	go install(Plugin{map[string]func() string{route: func() string { return "standard" }}}, standard, done)
	go install(Plugin{map[string]func() string{route: func() string { return "express" }}}, express, done)
	close(express)
	<-done
	close(standard)
	<-done
	fmt.Println("shipping", routes[route]())
}
