package main

import "fmt"

const defaultUserAgent = "defects/1.0"

func withDefaults(headers map[string]string) map[string]string {
	copy := make(map[string]string, len(headers))
	for k, v := range headers {
		copy[k] = v
	}
	headers = copy
	headers["User-Agent"] = defaultUserAgent
	return headers
}

func main() {
	signed := map[string]string{"Authorization": "Bearer secret"}
	public := withDefaults(signed)
	delete(public, "Authorization")
	fmt.Println("signed:", signed)
}
