package main

import "fmt"

const maxBatchItems = 3

func importRows(rows []string) (int, error) {
	if len(rows) > maxBatchItems {
		return 0, fmt.Errorf("batch rejected")
	}
	processed := 0
	for range rows {
		processed++
	}
	return processed, nil
}

func main() {
	processed, err := importRows([]string{"a", "b", "c", "d"})
	if err != nil {
		fmt.Println(err)
		return
	}
	fmt.Println("processed", processed)
}
