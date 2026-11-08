package main

import (
	"fmt"
	"math"
	"time"
)

func main() {
	requestedSeconds := int64(10_000_000_000)
	if requestedSeconds < 0 || requestedSeconds > math.MaxInt64/int64(time.Second) {
		fmt.Println("timeout rejected")
		return
	}
	timeout := time.Duration(requestedSeconds) * time.Second
	fmt.Println("timeout", timeout)
}
