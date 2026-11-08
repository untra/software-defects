package main

import (
	"fmt"
	"runtime"
	"time"
)

const requests = 100
const refreshInterval = time.Hour

func refreshSession(id int) {}

func handleLogin(id int) func() {
	done, stopped := make(chan struct{}), make(chan struct{})
	ticker := time.NewTicker(refreshInterval)
	go func() {
		defer close(stopped)
		defer ticker.Stop()
		for {
			select {
			case <-ticker.C:
				refreshSession(id)
			case <-done:
				return
			}
		}
	}()
	return func() { close(done); <-stopped }
}

func main() {
	before := runtime.NumGoroutine()
	for i := range requests {
		stop := handleLogin(i)
		stop()
	}
	fmt.Println("goroutines left running after", requests, "logins:", runtime.NumGoroutine()-before)
}
