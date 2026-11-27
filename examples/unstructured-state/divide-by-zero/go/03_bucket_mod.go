package main

import "fmt"

func main() {
	jobs := 10
	buckets := 0
	remainder := jobs % buckets
	fmt.Println("remainder", remainder)
}
