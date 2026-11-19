package main

import (
	"fmt"
	"maps"
)

func withAgent(headers map[string]string) map[string]string {
	return maps.Clone(headers)
}

func main() {
	signed := map[string]string{"Authorization": "Bearer secret"}
	public := withAgent(signed)
	delete(public, "Authorization")
	fmt.Println(signed)
}
