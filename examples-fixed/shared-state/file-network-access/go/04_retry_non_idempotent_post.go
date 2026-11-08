package main

import (
	"fmt"
	"net/http"
	"net/http/httptest"
	"strings"
	"sync"
	"sync/atomic"
	"time"
)

const clientTimeout = 50 * time.Millisecond
const slowFirstResponse = 100 * time.Millisecond
const maxAttempts = 3

func charge(client *http.Client, url string, operationID string) error {
	var err error
	for range maxAttempts {
		var resp *http.Response
		request, requestErr := http.NewRequest(http.MethodPost, url, strings.NewReader(`{"amount":50}`))
		if requestErr != nil {
			return requestErr
		}
		request.Header.Set("Idempotency-Key", operationID)
		resp, err = client.Do(request)
		if err == nil {
			resp.Body.Close()
			return nil
		}
	}
	return err
}

func main() {
	var charges atomic.Int32
	var processed sync.Map
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_, duplicate := processed.LoadOrStore(r.Header.Get("Idempotency-Key"), true)
		if !duplicate && charges.Add(1) == 1 {
			time.Sleep(slowFirstResponse)
		}
		w.WriteHeader(http.StatusCreated)
	}))
	client := &http.Client{Timeout: clientTimeout}
	err := charge(client, server.URL, "order-1")
	server.Close()
	fmt.Printf("charge error: %v, card charged %d times\n", err, charges.Load())
}
