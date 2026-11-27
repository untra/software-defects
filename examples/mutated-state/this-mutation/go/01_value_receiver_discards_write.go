package main

import "fmt"

const requestCount = 3

type Counter struct {
	hits int
}

func (c Counter) Record() {
	c.hits++
}

func main() {
	var c Counter
	for range requestCount {
		c.Record()
	}
	fmt.Println("hits:", c.hits)
}
