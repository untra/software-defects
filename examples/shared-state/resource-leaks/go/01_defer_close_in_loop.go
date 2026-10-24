package main

import (
	"fmt"
	"os"
	"path/filepath"
	"syscall"
)

const fileLimit = 32
const logFiles = 64
const fileMode = 0o644

func countLines(paths []string) (int, error) {
	total := 0
	for _, p := range paths {
		f, err := os.Open(p)
		if err != nil {
			return total, err
		}
		defer f.Close()
		total++
	}
	return total, nil
}

func main() {
	syscall.Setrlimit(syscall.RLIMIT_NOFILE, &syscall.Rlimit{Cur: fileLimit, Max: fileLimit})
	dir, _ := os.MkdirTemp("", "logs")
	defer os.RemoveAll(dir)
	var paths []string
	for i := range logFiles {
		p := filepath.Join(dir, fmt.Sprintf("app-%d.log", i))
		os.WriteFile(p, []byte("line\n"), fileMode)
		paths = append(paths, p)
	}
	n, err := countLines(paths)
	fmt.Printf("processed %d of %d files, error: %v\n", n, logFiles, err != nil)
}
