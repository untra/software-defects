package main

import (
	"fmt"
	"time"
)

func main() {
	issuedAt := time.Now()
	stored := issuedAt.Format(time.RFC3339Nano)
	loaded, err := time.Parse(time.RFC3339Nano, stored)
	if err != nil {
		panic(err)
	}
	fmt.Println("token matches:", issuedAt == loaded)
}
