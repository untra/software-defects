package main

import (
	"errors"
	"fmt"
	"runtime/debug"
)

const maxStackBytes = 1 << 20

var errUnavailable = errors.New("503 service unavailable")

func fetchInventory() ([]string, error) {
	return nil, errUnavailable
}

func fetchWithRetry() ([]string, error) {
	items, err := fetchInventory()
	if err != nil {
		return fetchWithRetry()
	}
	return items, nil
}

func main() {
	debug.SetMaxStack(maxStackBytes)
	items, err := fetchWithRetry()
	fmt.Println(items, err)
}
