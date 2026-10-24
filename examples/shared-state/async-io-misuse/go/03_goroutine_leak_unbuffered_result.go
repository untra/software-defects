package main

import (
	"fmt"
	"runtime"
	"time"
)

const searches = 10
const settle = 50 * time.Millisecond

var replicas = []string{"us-east", "us-west", "eu"}

func query(replica, term string) string {
	return replica + ":" + term
}

func firstResult(term string) string {
	results := make(chan string)
	for _, r := range replicas {
		go func() { results <- query(r, term) }()
	}
	return <-results
}

func main() {
	before := runtime.NumGoroutine()
	for range searches {
		firstResult("boots")
	}
	time.Sleep(settle)
	fmt.Println("goroutines leaked:", runtime.NumGoroutine()-before)
}
