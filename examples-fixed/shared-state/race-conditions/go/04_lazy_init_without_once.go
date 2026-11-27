package main

import (
	"fmt"
	"sync"
	"sync/atomic"
	"time"
)

const callers = 8
const loadLatency = 20 * time.Millisecond

var registryOnce sync.Once
var registry map[string]int
var loads atomic.Int32

func getRegistry() map[string]int {
	registryOnce.Do(func() {
		time.Sleep(loadLatency)
		loads.Add(1)
		registry = map[string]int{"plugins": 3}
	})
	return registry
}

func main() {
	var wg sync.WaitGroup
	for range callers {
		wg.Add(1)
		go func() {
			defer wg.Done()
			getRegistry()
		}()
	}
	wg.Wait()
	fmt.Println("registry loaded", loads.Load(), "times")
}
