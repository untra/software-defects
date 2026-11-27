package main

import (
	"fmt"
	"io"
)

const chunkBytes = 4

var records = []string{}

func receive(reader io.Reader) {
	chunk := make([]byte, chunkBytes)
	for {
		n, err := reader.Read(chunk)
		if n > 0 {
			records = append(records, string(chunk[:n]))
		}
		if err == io.EOF {
			return
		}
		if err != nil {
			panic(err)
		}
	}
}
func main() {
	reader, writer := io.Pipe()
	go func() {
		defer writer.Close()
		names := []string{"pears", "figs-large"}
		var bytes []byte
		for _, name := range names {
			bytes = append(bytes, []byte(name)...)
		}
		if _, err := writer.Write(bytes); err != nil {
			panic(err)
		}
	}()
	receive(reader)
	fmt.Printf("records %q\n", records)
}
