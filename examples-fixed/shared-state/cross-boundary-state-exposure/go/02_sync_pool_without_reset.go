package main

import (
	"bytes"
	"fmt"
	"sync"
)

var bufferPool = sync.Pool{New: func() any { return new(bytes.Buffer) }}

func renderReceipt(user, card string) string {
	buf := bufferPool.Get().(*bytes.Buffer)
	buf.Reset()
	defer func() { buf.Reset(); bufferPool.Put(buf) }()
	fmt.Fprintf(buf, "receipt for %s, card ending %s\n", user, card)
	return buf.String()
}

func main() {
	renderReceipt("alice", "4242")
	fmt.Print(renderReceipt("bob", "1881"))
}
