package main

import "fmt"

type Server struct {
	Name    string
	Metrics map[string]int
}

func main() {
	primary := Server{Name: "primary", Metrics: map[string]int{"hits": 0}}
	replica := primary
	replica.Metrics["hits"] = 1
	fmt.Println(primary.Metrics["hits"])
}
