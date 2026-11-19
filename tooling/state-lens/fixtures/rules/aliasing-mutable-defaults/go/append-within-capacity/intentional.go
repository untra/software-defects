package main

import "fmt"

func main() {
	data := make([]int, 0, 4)
	view := data[:0]
	view = append(view, 7)
	fmt.Println(view, data[:1])
}
