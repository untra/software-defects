package main

import (
	"fmt"
	"sync"
)

const workers = 8
const loginsPerWorker = 100_000

var sessions = map[int]string{}

func login(userID int) {
	sessions[userID] = "active"
}

func main() {
	var wg sync.WaitGroup
	for w := range workers {
		wg.Add(1)
		go func() {
			defer wg.Done()
			for i := range loginsPerWorker {
				login(w*loginsPerWorker + i)
			}
		}()
	}
	wg.Wait()
	fmt.Println("sessions:", len(sessions))
}
