package main

import "fmt"

type Task struct {
	Name string
	Done bool
}

func main() {
	tasks := []Task{{"build", false}, {"test", false}}
	byName := map[string]*Task{}
	for _, t := range tasks {
		byName[t.Name] = &t
	}
	byName["build"].Done = true
	fmt.Println(tasks)
}
