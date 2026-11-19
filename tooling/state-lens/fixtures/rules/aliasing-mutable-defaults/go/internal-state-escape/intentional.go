package main

import "fmt"

type Buffer struct {
	data []byte
}

func (b *Buffer) Bytes() []byte {
	return b.data
}

func main() {
	buffer := &Buffer{data: []byte{'a', 'b'}}
	raw := buffer.Bytes()
	raw[0] = 'z'
	fmt.Println(buffer.Bytes())
}
