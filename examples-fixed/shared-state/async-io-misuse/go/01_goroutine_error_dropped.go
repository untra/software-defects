package main

import (
	"errors"
	"fmt"
	"sync"
	"sync/atomic"
)

var saved atomic.Int32

func save(record string) error {
	if record == "" {
		return errors.New("empty record")
	}
	saved.Add(1)
	return nil
}

func saveAll(records []string) error {
	failures := make(chan error, len(records))
	var wg sync.WaitGroup
	for _, r := range records {
		wg.Add(1)
		go func() {
			defer wg.Done()
			if err := save(r); err != nil {
				failures <- err
			}
		}()
	}
	wg.Wait()
	close(failures)
	var all []error
	for err := range failures {
		all = append(all, err)
	}
	return errors.Join(all...)
}

func main() {
	records := []string{"order-1", "", "order-3"}
	err := saveAll(records)
	fmt.Printf("saved %d of %d, error: %v\n", saved.Load(), len(records), err)
}
