package main

import (
	"errors"
	"fmt"
	"runtime/debug"
)

const maxAttempts = 3
const maxStackBytes = 1 << 20

var errUnavailable = errors.New("503 service unavailable")

func fetchInventory() ([]string, error) {
	return nil, errUnavailable
}

func fetchWithRetry() ([]string, error) {
	var lastErr error
	for range maxAttempts {
		items, err := fetchInventory()
		if err == nil {
			return items, nil
		}
		lastErr = err
	}
	return nil, lastErr
}

func main() {
	debug.SetMaxStack(maxStackBytes)
	items, err := fetchWithRetry()
	fmt.Println(items, err)
}
