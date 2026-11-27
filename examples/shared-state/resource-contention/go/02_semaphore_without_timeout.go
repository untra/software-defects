package main

import (
	"fmt"
	"sync"
	"time"
)

const workerSlots = 2
const reportDuration = 100 * time.Millisecond
const healthBudget = 10 * time.Millisecond
const reportStart = 5 * time.Millisecond

var slots = make(chan struct{}, workerSlots)

func withSlot(work func()) {
	slots <- struct{}{}
	defer func() { <-slots }()
	work()
}

func main() {
	var wg sync.WaitGroup
	for range workerSlots {
		wg.Add(1)
		go func() {
			defer wg.Done()
			withSlot(func() { time.Sleep(reportDuration) })
		}()
	}
	time.Sleep(reportStart)
	start := time.Now()
	withSlot(func() {})
	waited := time.Since(start)
	wg.Wait()
	fmt.Println("health check within budget:", waited <= healthBudget)
}
