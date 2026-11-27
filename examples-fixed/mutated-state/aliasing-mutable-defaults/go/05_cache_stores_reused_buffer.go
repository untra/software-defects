package main

import (
	"fmt"
	"strings"
)

const bufferSize = 64

func main() {
	cache := map[string][]byte{}
	buf := make([]byte, 0, bufferSize)
	for _, key := range []string{"alpha", "beta"} {
		buf = buf[:0]
		buf = append(buf, strings.ToUpper(key)...)
		cache[key] = append([]byte(nil), buf...)
	}
	fmt.Println(string(cache["alpha"]), string(cache["beta"]))
}
