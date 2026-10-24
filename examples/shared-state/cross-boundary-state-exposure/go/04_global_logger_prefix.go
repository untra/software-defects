package main

import (
	"log"
	"os"
	"sync"
	"time"
)

const chargeLatency = 20 * time.Millisecond
const requestStagger = 5 * time.Millisecond

func handleCharge(requestID string) {
	log.SetPrefix("[" + requestID + "] ")
	time.Sleep(chargeLatency)
	log.Println("card charged")
}

func main() {
	log.SetOutput(os.Stdout)
	log.SetFlags(0)
	var wg sync.WaitGroup
	for _, id := range []string{"req-1", "req-2"} {
		wg.Add(1)
		go func() {
			defer wg.Done()
			handleCharge(id)
		}()
		time.Sleep(requestStagger)
	}
	wg.Wait()
}
