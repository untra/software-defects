package main

import "fmt"

func save(path string) error {
	return fmt.Errorf("disk full: %s", path)
}

func main() {
	if err := save("invoice.txt"); err != nil {
		fmt.Println("save failed:", err)
		return
	}
	fmt.Println("saved invoice.txt")
}
