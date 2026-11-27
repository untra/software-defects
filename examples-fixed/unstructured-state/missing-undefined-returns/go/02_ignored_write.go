package main

import "fmt"

func writeInvoice(path string) error { return fmt.Errorf("disk full: %s", path) }
func save(path string) error {
	if err := writeInvoice(path); err != nil {
		return err
	}
	return nil
}
func main() { fmt.Printf("save error=%v\n", save("invoice.txt")) }
