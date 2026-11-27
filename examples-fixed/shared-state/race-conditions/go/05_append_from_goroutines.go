package main

import (
	"fmt"
	"sync"
)

const shards = 8
const rowsPerShard = 10_000

func collect() []int {
	var rows []int
	var mu sync.Mutex
	var wg sync.WaitGroup
	for s := range shards {
		wg.Add(1)
		go func() {
			defer wg.Done()
			for i := range rowsPerShard {
				mu.Lock()
				rows = append(rows, s*rowsPerShard+i)
				mu.Unlock()
			}
		}()
	}
	wg.Wait()
	return rows
}

func main() {
	fmt.Printf("collected %d of %d rows\n", len(collect()), shards*rowsPerShard)
}
