package main

import (
	"fmt"
	"time"
)

const flushLatency = 50 * time.Millisecond

func flushMetrics() {
	time.Sleep(flushLatency)
	fmt.Println("metrics flushed")
}

func shutdown() {
	go flushMetrics()
	fmt.Println("shutdown complete")
}

func main() {
	shutdown()
}
