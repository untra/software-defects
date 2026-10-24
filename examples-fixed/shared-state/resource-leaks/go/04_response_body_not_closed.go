package main

import (
	"fmt"
	"io"
	"net"
	"net/http"
	"net/http/httptest"
	"sync/atomic"
)

const healthChecks = 20

func isHealthy(client *http.Client, url string) bool {
	resp, err := client.Get(url)
	if err != nil {
		return false
	}
	defer resp.Body.Close()
	if _, err := io.Copy(io.Discard, resp.Body); err != nil {
		return false
	}
	return resp.StatusCode == http.StatusOK
}

func main() {
	var connections atomic.Int32
	server := httptest.NewUnstartedServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		fmt.Fprint(w, "ok")
	}))
	server.Config.ConnState = func(c net.Conn, s http.ConnState) {
		if s == http.StateNew {
			connections.Add(1)
		}
	}
	server.Start()
	defer server.Close()
	client := server.Client()
	for range healthChecks {
		isHealthy(client, server.URL)
	}
	fmt.Printf("%d health checks opened %d connections\n", healthChecks, connections.Load())
}
