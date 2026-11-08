package main

import "fmt"

type Task struct {
	ID   int
	Name string
}

func buildTaskRegistry(names []string) []*Task {
	var tasks []*Task
	for i, name := range names {
		t := Task{ID: i, Name: name}
		tasks = append(tasks, &t)
	}
	return tasks
}

func main() {
	tasks := buildTaskRegistry([]string{"first", "second"})
	for _, task := range tasks {
		fmt.Println(task.ID, task.Name)
	}
	fmt.Println("distinct:", tasks[0] != tasks[1])
}
