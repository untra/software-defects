package main

import "fmt"

func main() {
	jobs := []string{"resize", "encode", "upload", "notify"}
	batch := jobs[:2]
	batch = append([]string(nil), batch...)
	batch = append(batch, "retry")
	fmt.Println(batch, jobs)
}
