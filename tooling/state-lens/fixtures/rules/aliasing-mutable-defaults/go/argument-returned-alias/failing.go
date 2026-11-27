package main

import "fmt"

func withAgent(headers map[string]string) map[string]string {
	return headers
}

func main() {
	signed := map[string]string{"Authorization": "Bearer secret"}
	public := withAgent(signed)
	delete(public, "Authorization")
	fmt.Println(signed)
}
