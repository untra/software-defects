package main

import "fmt"

func headersFor(token string) map[string]string {
	return map[string]string{"Authorization": token}
}

func main() {
	first := headersFor("a")
	second := headersFor("b")
	delete(first, "Authorization")
	fmt.Println(second)
}
