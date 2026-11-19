package main

import "fmt"

func main() {
	jobs := []string{"resize", "encode"}
	batch := []string{"resize", "encode"}
	batch = append(batch, "retry")
	fmt.Println(batch, jobs)
}
