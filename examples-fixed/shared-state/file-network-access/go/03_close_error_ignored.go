package main

import (
	"errors"
	"fmt"
	"io"
)

type BufferedUpload struct {
	pending []byte
}

func (u *BufferedUpload) Write(p []byte) (int, error) {
	u.pending = append(u.pending, p...)
	return len(p), nil
}

func (u *BufferedUpload) Close() error {
	return errors.New("flush failed: quota exceeded")
}

func openUpload(name string) io.WriteCloser {
	return &BufferedUpload{}
}

func backup(name string, data []byte) error {
	w := openUpload(name)
	_, err := w.Write(data)
	return errors.Join(err, w.Close())
}

func main() {
	err := backup("db.dump", []byte("rows"))
	fmt.Println("backup error:", err)
}
