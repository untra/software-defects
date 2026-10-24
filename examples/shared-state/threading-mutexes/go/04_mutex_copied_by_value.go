package main

import (
	"fmt"
	"sync"
)

const workers = 8
const signupsPerWorker = 10_000

type Registry struct {
	mu    sync.Mutex
	users map[int]bool
}

func (r Registry) Add(id int) {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.users[id] = true
}

func main() {
	registry := Registry{users: map[int]bool{}}
	var wg sync.WaitGroup
	for w := range workers {
		wg.Add(1)
		go func() {
			defer wg.Done()
			for i := range signupsPerWorker {
				registry.Add(w*signupsPerWorker + i)
			}
		}()
	}
	wg.Wait()
	fmt.Println("registered users:", len(registry.users))
}
