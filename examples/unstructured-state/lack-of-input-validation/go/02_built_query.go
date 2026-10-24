package main

import "fmt"

func main() {
	name := "O'Brien"
	query := fmt.Sprintf("SELECT id FROM users WHERE name = '%s'", name)
	fmt.Println(query)
}
