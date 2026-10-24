package main

import "fmt"

type Task struct {
	Name string
	Done bool
}

func main() {
	tasks := []Task{{"build", false}, {"test", false}}
	byName := map[string]*Task{}
	for i := range tasks {
		byName[tasks[i].Name] = &tasks[i]
	}
	byName["build"].Done = true
	fmt.Println(tasks)
}
