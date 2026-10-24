package main

import (
	"fmt"
	"sync"
)

const workers = 8
const viewsPerWorker = 1_000_000

type PageViews struct {
	mu    sync.Mutex
	total int
}

func (p *PageViews) Record() {
	p.mu.Lock()
	defer p.mu.Unlock()
	p.total++
}

func main() {
	var views PageViews
	var wg sync.WaitGroup
	for range workers {
		wg.Add(1)
		go func() {
			defer wg.Done()
			for range viewsPerWorker {
				views.Record()
			}
		}()
	}
	wg.Wait()
	fmt.Printf("recorded %d of %d views\n", views.total, workers*viewsPerWorker)
}
