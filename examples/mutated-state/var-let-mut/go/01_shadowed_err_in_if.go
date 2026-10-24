package main

import (
	"errors"
	"fmt"
	"strconv"
)

func parsePort(raw string) (int, error) {
	var port int
	var err error
	if raw != "" {
		port, err := strconv.Atoi(raw)
		if err == nil && port > 0 {
			fmt.Println("parsed port", port)
		}
	}
	if port == 0 && err == nil {
		err = errors.New("port not set")
	}
	return port, err
}

func main() {
	port, err := parsePort("8080")
	fmt.Printf("port=%d err=%v\n", port, err)
}
