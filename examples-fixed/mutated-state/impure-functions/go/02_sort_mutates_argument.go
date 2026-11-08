package main

import (
	"fmt"
	"sort"
)

func median(latencies []int) int {
	latencies = append([]int(nil), latencies...)
	sort.Ints(latencies)
	return latencies[len(latencies)/2]
}

func main() {
	latenciesByArrival := []int{30, 10, 20}
	m := median(latenciesByArrival)
	fmt.Println("median:", m, "first request latency:", latenciesByArrival[0])
}
