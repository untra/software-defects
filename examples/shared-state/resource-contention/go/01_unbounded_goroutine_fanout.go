package main

import (
	"fmt"
	"sync"
	"sync/atomic"
	"time"
)

const maxConnections = 20
const userCount = 1000
const queryLatency = 20 * time.Millisecond

type Database struct {
	open     atomic.Int32
	rejected atomic.Int32
}

func (db *Database) Query(id int) {
	if db.open.Add(1) > maxConnections {
		db.rejected.Add(1)
	}
	time.Sleep(queryLatency)
	db.open.Add(-1)
}

func refreshAll(db *Database, ids []int) {
	var wg sync.WaitGroup
	for _, id := range ids {
		wg.Add(1)
		go func() {
			defer wg.Done()
			db.Query(id)
		}()
	}
	wg.Wait()
}

func main() {
	db := &Database{}
	ids := make([]int, userCount)
	for i := range ids {
		ids[i] = i
	}
	refreshAll(db, ids)
	fmt.Println("queries rejected by connection limit:", db.rejected.Load() > 0)
}
