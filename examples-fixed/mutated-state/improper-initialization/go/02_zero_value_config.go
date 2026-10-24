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

const defaultTimeout = 5 * time.Second
const defaultRetries = 3

func main() {
	cfg := ClientConfig{BaseURL: "https://api.example.com", Timeout: defaultTimeout, MaxRetries: defaultRetries}
	fmt.Println("timeout:", cfg.Timeout, "retries:", cfg.MaxRetries)
}
