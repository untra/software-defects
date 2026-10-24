package main

import "fmt"

const quotaKB int64 = 30_000

func main() {
	uploadsKB := []int64{20_000, 20_000}
	var totalKB int64
	for _, kb := range uploadsKB {
		totalKB += kb
	}
	if totalKB > quotaKB {
		fmt.Println("rejected: over quota")
	} else {
		fmt.Println("accepted, total KB:", totalKB)
	}
}
