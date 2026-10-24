package main

import (
	"fmt"
	"os"
	"syscall"
	"unsafe"
)

const fileMode = 0o600
const recordStart = 1
const missingIndex = -1
const replacement = 'X'

var fileBody = []byte{'H', 'A', 'B'}

func locate(records []byte, key byte) int {
	for i, value := range records {
		if value == key {
			return i
		}
	}
	return missingIndex
}
func main() {
	file, err := os.CreateTemp("", "records-")
	if err != nil {
		panic(err)
	}
	defer os.Remove(file.Name())
	defer file.Close()
	if _, err := file.Write(fileBody); err != nil {
		panic(err)
	}
	page, err := syscall.Mmap(int(file.Fd()), 0, len(fileBody), syscall.PROT_READ|syscall.PROT_WRITE, syscall.MAP_SHARED)
	if err != nil {
		panic(err)
	}
	records := page[recordStart:]
	index := locate(records, 'Z')
	*(*byte)(unsafe.Add(unsafe.Pointer(&records[0]), index)) = replacement
	if err := syscall.Munmap(page); err != nil {
		panic(err)
	}
	body, err := os.ReadFile(file.Name())
	if err != nil {
		panic(err)
	}
	fmt.Println("header", string(body[:recordStart]))
}
