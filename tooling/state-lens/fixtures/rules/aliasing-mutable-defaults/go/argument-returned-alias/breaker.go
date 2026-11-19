package main

import (
	"fmt"
	"maps"
)

func withAgent(headers map[string]string) map[string]string {
	return headers
}

func main() {
	signed := map[string]string{"Authorization": "Bearer secret"}
	public := maps.Clone(withAgent(signed))
	delete(public, "Authorization")
	fmt.Println(signed)
}
