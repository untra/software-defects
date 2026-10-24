package main

import (
	"fmt"
	"os"
	"path/filepath"
)

const stamp = 1700000000000
const prefix = "invoice-"
const fileMode = 0o600

func save(path string, checked chan<- struct{}, proceed <-chan struct{}, done chan<- struct{}) {
	if _, err := os.Stat(path); os.IsNotExist(err) {
		close(checked)
		<-proceed
		if err := os.WriteFile(path, []byte("orchard"), fileMode); err != nil {
			panic(err)
		}
	}
	close(done)
}
func main() {
	directory, err := os.MkdirTemp("", prefix)
	if err != nil {
		panic(err)
	}
	defer os.RemoveAll(directory)
	path := filepath.Join(directory, fmt.Sprint(prefix, stamp))
	checked, proceed, done := make(chan struct{}), make(chan struct{}), make(chan struct{})
	go save(path, checked, proceed, done)
	<-checked
	if err := os.WriteFile(path, []byte("harbor"), fileMode); err != nil {
		panic(err)
	}
	close(proceed)
	<-done
	body, err := os.ReadFile(path)
	if err != nil {
		panic(err)
	}
	fmt.Println("existing document", string(body))
}
