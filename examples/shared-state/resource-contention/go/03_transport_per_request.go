package main

import (
	"fmt"
	"io"
	"net"
	"net/http"
	"net/http/httptest"
	"sync/atomic"
)

const requests = 20

func fetchPrice(url string) (string, error) {
	client := &http.Client{Transport: &http.Transport{}}
	resp, err := client.Get(url)
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()
	body, err := io.ReadAll(resp.Body)
	return string(body), err
}

func main() {
	var connections atomic.Int32
	server := httptest.NewUnstartedServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		fmt.Fprint(w, "42.00")
	}))
	server.Config.ConnState = func(c net.Conn, s http.ConnState) {
		if s == http.StateNew {
			connections.Add(1)
		}
	}
	server.Start()
	defer server.Close()
	for range requests {
		if _, err := fetchPrice(server.URL); err != nil {
			panic(err)
		}
	}
	fmt.Printf("%d requests opened %d connections\n", requests, connections.Load())
}
