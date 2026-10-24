package main

import "fmt"

const maxPreviewIndex = 10

func lastIndex(items []string) uint {
	return uint(len(items)) - 1
}

func main() {
	var drafts []string
	idx := lastIndex(drafts)
	fmt.Println("last index:", idx, "within preview:", idx < maxPreviewIndex)
}
