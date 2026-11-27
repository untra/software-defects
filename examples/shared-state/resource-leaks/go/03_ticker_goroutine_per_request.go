package main

import (
	"fmt"
	"runtime"
	"time"
)

const requests = 100
const refreshInterval = time.Hour

func refreshSession(id int) {}

func handleLogin(id int) {
	go func() {
		for range time.Tick(refreshInterval) {
			refreshSession(id)
		}
	}()
}

func main() {
	before := runtime.NumGoroutine()
	for i := range requests {
		handleLogin(i)
	}
	fmt.Println("goroutines left running after", requests, "logins:", runtime.NumGoroutine()-before)
}
