package main

import "fmt"

const batchSize = 2

func main() {
	jobs := []string{"resize", "encode", "upload", "notify"}
	firstBatch := jobs[:batchSize]
	firstBatch = append(firstBatch, "urgent-retry")
	fmt.Println("batch:", firstBatch)
	fmt.Println("jobs: ", jobs)
}
