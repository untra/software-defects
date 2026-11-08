package main

import (
	"fmt"
	"net/http"
	"net/http/httptest"
	"strings"
	"sync/atomic"
	"time"
)

const clientTimeout = 50 * time.Millisecond
const slowFirstResponse = 100 * time.Millisecond
const maxAttempts = 3

func charge(client *http.Client, url string) error {
	var err error
	for range maxAttempts {
		var resp *http.Response
		resp, err = client.Post(url, "application/json", strings.NewReader(`{"amount":50}`))
		if err == nil {
			resp.Body.Close()
			return nil
		}
	}
	return err
}

func main() {
	var charges atomic.Int32
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if charges.Add(1) == 1 {
			time.Sleep(slowFirstResponse)
		}
		w.WriteHeader(http.StatusCreated)
	}))
	client := &http.Client{Timeout: clientTimeout}
	err := charge(client, server.URL)
	server.Close()
	fmt.Printf("charge error: %v, card charged %d times\n", err, charges.Load())
}
