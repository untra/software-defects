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

func withSlot(work func()) bool {
	timer := time.NewTimer(healthBudget)
	defer timer.Stop()
	select {
	case slots <- struct{}{}:
	case <-timer.C:
		return false
	}
	defer func() { <-slots }()
	work()
	return true
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
	accepted := withSlot(func() {})
	waited := time.Since(start)
	wg.Wait()
	fmt.Println("health check within budget:", !accepted || waited <= healthBudget)
}
