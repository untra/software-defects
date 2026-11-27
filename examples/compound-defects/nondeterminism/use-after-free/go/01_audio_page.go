package main

import (
	"fmt"
	"runtime/debug"
	"syscall"
	"unsafe"
)

func main() {
	page, err := syscall.Mmap(-1, 0, syscall.Getpagesize(), syscall.PROT_READ|syscall.PROT_WRITE, syscall.MAP_ANON|syscall.MAP_PRIVATE)
	if err != nil {
		panic(err)
	}
	playback := (*byte)(unsafe.Pointer(&page[0]))
	if err := syscall.Munmap(page); err != nil {
		panic(err)
	}
	debug.SetPanicOnFault(true)
	fmt.Println("sample", *playback)
}
