package main

import (
	"fmt"
	"time"
)

type ClientConfig struct {
	BaseURL    string
	Timeout    time.Duration
	MaxRetries int
}

func main() {
	cfg := ClientConfig{BaseURL: "https://api.example.com"}
	fmt.Println("timeout:", cfg.Timeout, "retries:", cfg.MaxRetries)
}
