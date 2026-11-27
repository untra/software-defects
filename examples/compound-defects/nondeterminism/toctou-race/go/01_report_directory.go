package main

import (
	"fmt"
	"os"
	"path/filepath"
)

const directoryMode = 0o700

func initialize(path string, checked, proceed chan struct{}) error {
	if _, err := os.Stat(path); os.IsNotExist(err) {
		close(checked)
		<-proceed
		return os.Mkdir(path, directoryMode)
	}
	return nil
}
func main() {
	directory, err := os.MkdirTemp("", "reports-")
	if err != nil {
		panic(err)
	}
	defer os.RemoveAll(directory)
	path := filepath.Join(directory, "daily")
	checked, proceed := make(chan struct{}), make(chan struct{})
	result := make(chan error, 1)
	go func() { result <- initialize(path, checked, proceed) }()
	<-checked
	if err := os.Mkdir(path, directoryMode); err != nil {
		panic(err)
	}
	close(proceed)
	fmt.Println("directory already occupied", os.IsExist(<-result))
}
