package main

import "fmt"

func save(path string) error {
	return fmt.Errorf("disk full: %s", path)
}

func main() {
	_ = save("invoice.txt")
	fmt.Println("saved invoice.txt")
}
