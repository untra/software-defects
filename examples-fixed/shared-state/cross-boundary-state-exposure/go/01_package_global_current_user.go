package main

import (
	"fmt"
	"sort"
	"sync"
	"time"
)

const renderLatency = 20 * time.Millisecond
const requestStagger = 5 * time.Millisecond

func handleDashboard(user string) string {
	time.Sleep(renderLatency)
	return fmt.Sprintf("%s received account data of %s", user, user)
}

func main() {
	var mu sync.Mutex
	var responses []string
	var wg sync.WaitGroup
	for _, user := range []string{"alice", "bob"} {
		wg.Add(1)
		go func() {
			defer wg.Done()
			r := handleDashboard(user)
			mu.Lock()
			responses = append(responses, r)
			mu.Unlock()
		}()
		time.Sleep(requestStagger)
	}
	wg.Wait()
	sort.Strings(responses)
	for _, r := range responses {
		fmt.Println(r)
	}
}
