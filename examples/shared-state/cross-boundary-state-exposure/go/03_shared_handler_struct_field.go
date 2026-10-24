package main

import (
	"fmt"
	"sort"
	"sync"
	"time"
)

const queryLatency = 20 * time.Millisecond
const requestStagger = 5 * time.Millisecond

type ReportHandler struct {
	tenant string
}

func (h *ReportHandler) Serve(tenant string) string {
	h.tenant = tenant
	time.Sleep(queryLatency)
	return fmt.Sprintf("%s got rows for tenant %s", tenant, h.tenant)
}

func main() {
	handler := &ReportHandler{}
	var mu sync.Mutex
	var responses []string
	var wg sync.WaitGroup
	for _, tenant := range []string{"acme", "globex"} {
		wg.Add(1)
		go func() {
			defer wg.Done()
			r := handler.Serve(tenant)
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
