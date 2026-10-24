package main

import (
	"fmt"
	"runtime"
	"sync"
	"time"
)

const publishWindow = 50 * time.Millisecond
const pollingLimit = time.Second

type Service struct {
	name  string
	ready int
}

var (
	current *Service
	mu      sync.Mutex
)

func instance() *Service {
	if current == nil {
		mu.Lock()
		if current == nil {
			service := &Service{}
			current = service
			time.Sleep(publishWindow)
			service.name = "billing"
			service.ready = 1
		}
		mu.Unlock()
	}
	return current
}

func main() {
	go instance()
	var seen *Service
	deadline := time.Now().Add(pollingLimit)
	for seen == nil && time.Now().Before(deadline) {
		seen = current
		runtime.Gosched()
	}
	if seen == nil {
		fmt.Println("service unavailable")
		return
	}
	fmt.Printf("name %q ready %d\n", seen.name, seen.ready)
}
