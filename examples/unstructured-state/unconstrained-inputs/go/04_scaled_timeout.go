package main

import (
	"fmt"
	"time"
)

func main() {
	requestedSeconds := int64(10_000_000_000)
	timeout := time.Duration(requestedSeconds) * time.Second
	fmt.Println("timeout", timeout)
}
