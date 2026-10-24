package main

import (
	"fmt"
	"runtime/debug"
)

const maxStackBytes = 1 << 20

var dependencies = map[string][]string{
	"app":    {"auth", "db"},
	"auth":   {"crypto"},
	"db":     {"driver"},
	"driver": {"app"},
	"crypto": {},
}

func installOrder(pkg string) ([]string, error) {
	visiting, visited := map[string]bool{}, map[string]bool{}
	var order []string
	var visit func(string) error
	visit = func(name string) error {
		if visiting[name] {
			return fmt.Errorf("dependency cycle at %s", name)
		}
		if visited[name] {
			return nil
		}
		visiting[name] = true
		for _, dependency := range dependencies[name] {
			if err := visit(dependency); err != nil {
				return err
			}
		}
		delete(visiting, name)
		visited[name] = true
		order = append(order, name)
		return nil
	}
	err := visit(pkg)
	return order, err
}

func main() {
	debug.SetMaxStack(maxStackBytes)
	fmt.Println(installOrder("app"))
}
