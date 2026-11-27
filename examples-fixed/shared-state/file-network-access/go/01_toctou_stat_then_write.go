package main

import (
	"fmt"
	"os"
	"path/filepath"
	"sync"
	"sync/atomic"
	"time"
)

const claimLatency = 10 * time.Millisecond
const lockFileMode = 0o644

func claimLeadership(lockPath, node string) bool {
	f, err := os.OpenFile(lockPath, os.O_WRONLY|os.O_CREATE|os.O_EXCL, lockFileMode)
	if err != nil {
		return false
	}
	_, writeErr := f.WriteString(node)
	closeErr := f.Close()
	return writeErr == nil && closeErr == nil
}

func main() {
	dir, _ := os.MkdirTemp("", "leader")
	defer os.RemoveAll(dir)
	lockPath := filepath.Join(dir, "leader.lock")
	var leaders atomic.Int32
	var wg sync.WaitGroup
	for _, node := range []string{"node-a", "node-b", "node-c"} {
		wg.Add(1)
		go func() {
			defer wg.Done()
			if claimLeadership(lockPath, node) {
				leaders.Add(1)
			}
		}()
	}
	wg.Wait()
	fmt.Println("nodes that believe they are leader:", leaders.Load())
}
