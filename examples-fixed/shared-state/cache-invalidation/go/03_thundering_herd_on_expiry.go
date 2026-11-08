package main

import (
	"fmt"
	"sync"
	"sync/atomic"
	"time"
)

const concurrentReaders = 50
const loadLatency = 20 * time.Millisecond
const ttl = time.Minute

type entry struct {
	value   string
	expires time.Time
}

type Cache struct {
	mu      sync.Mutex
	entries map[string]entry
	loads   atomic.Int32
}

func (c *Cache) Get(key string) string {
	c.mu.Lock()
	defer c.mu.Unlock()
	e, ok := c.entries[key]
	if ok && time.Now().Before(e.expires) {
		return e.value
	}
	c.loads.Add(1)
	time.Sleep(loadLatency)
	value := "homepage for " + key
	c.entries[key] = entry{value: value, expires: time.Now().Add(ttl)}
	return value
}

func main() {
	cache := &Cache{entries: map[string]entry{}}
	var wg sync.WaitGroup
	for range concurrentReaders {
		wg.Add(1)
		go func() {
			defer wg.Done()
			cache.Get("en-US")
		}()
	}
	wg.Wait()
	fmt.Println("origin loads for one key:", cache.loads.Load())
}
