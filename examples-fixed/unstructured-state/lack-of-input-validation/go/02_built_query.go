package main

import "fmt"

func main() {
	name := "O'Brien"
	const query = "SELECT id FROM users WHERE name = ?"
	args := []any{name}
	fmt.Println(query, args)
}
