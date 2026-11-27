package main

import (
	"fmt"
	"os"
	"path/filepath"
)

const customerName = "O'Neil"
const fileMode = 0o600

func main() {
	directory, err := os.MkdirTemp("", "customers-")
	if err != nil {
		panic(err)
	}
	defer os.RemoveAll(directory)
	path := filepath.Join(directory, "name.txt")
	if err := os.WriteFile(path, []byte(customerName), fileMode); err != nil {
		panic(err)
	}
	body, err := os.ReadFile(path)
	if err != nil {
		panic(err)
	}
	query := "SELECT id FROM customers WHERE name = '" + string(body) + "'"
	fmt.Println(query)
}
