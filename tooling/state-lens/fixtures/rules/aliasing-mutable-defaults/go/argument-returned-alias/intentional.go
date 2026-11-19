package main

import "fmt"

func chain(headers map[string]string) map[string]string {
	return headers
}

func main() {
	request := map[string]string{}
	chain(request)["Accept"] = "application/json"
	fmt.Println(request)
}
