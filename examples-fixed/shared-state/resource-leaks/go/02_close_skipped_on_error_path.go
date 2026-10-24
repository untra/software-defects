package main

import (
	"errors"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"syscall"
)

const fileLimit = 32
const attempts = 64
const headerSize = 16
const fileMode = 0o644

var errShortHeader = errors.New("short header")

func readHeader(path string) ([]byte, error) {
	f, err := os.Open(path)
	if err != nil {
		return nil, err
	}
	defer f.Close()
	header := make([]byte, headerSize)
	if _, err := io.ReadFull(f, header); err != nil {
		return nil, errShortHeader
	}
	return header, nil
}

func main() {
	syscall.Setrlimit(syscall.RLIMIT_NOFILE, &syscall.Rlimit{Cur: fileLimit, Max: fileLimit})
	dir, _ := os.MkdirTemp("", "uploads")
	defer os.RemoveAll(dir)
	path := filepath.Join(dir, "truncated.bin")
	os.WriteFile(path, []byte("tiny"), fileMode)
	for i := range attempts {
		if _, err := readHeader(path); !errors.Is(err, errShortHeader) {
			fmt.Printf("attempt %d failed with too many open files: %v\n", i+1, errors.Is(err, syscall.EMFILE))
			return
		}
	}
	fmt.Println("all attempts reported short header")
}
