package main

import "fmt"

type Policy interface {
	Allow(requests int) bool
}

type Limiter struct {
	maxPerMinute int
}

func (l Limiter) Allow(requests int) bool { return requests <= l.maxPerMinute }

func main() {
	limiter := Limiter{maxPerMinute: 10}
	var policy Policy = &limiter
	limiter.maxPerMinute = 100
	fmt.Println("allow 50:", policy.Allow(50))
}
