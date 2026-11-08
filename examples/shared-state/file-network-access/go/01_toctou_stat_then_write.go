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
	if _, err := os.Stat(lockPath); os.IsNotExist(err) {
		time.Sleep(claimLatency)
		return os.WriteFile(lockPath, []byte(node), lockFileMode) == nil
	}
	return false
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
