package main

import (
	"fmt"
	"time"
)

const auditWrite = 30 * time.Millisecond
const requests = 5
const requestBudget = 10 * time.Millisecond

var auditEvents = make(chan string)

func auditor() {
	for range auditEvents {
		time.Sleep(auditWrite)
	}
}

func handleRequest(id int) {
	auditEvents <- fmt.Sprintf("request %d served", id)
}

func main() {
	go auditor()
	slowest := time.Duration(0)
	for i := range requests {
		start := time.Now()
		handleRequest(i)
		slowest = max(slowest, time.Since(start))
	}
	fmt.Println("all requests within budget:", slowest <= requestBudget)
}
