package main

import "fmt"

func main() {
	jobs := 10
	buckets := 0
	if buckets == 0 {
		fmt.Println("zero divisor")
		return
	}
	remainder := jobs % buckets
	fmt.Println("remainder", remainder)
}
