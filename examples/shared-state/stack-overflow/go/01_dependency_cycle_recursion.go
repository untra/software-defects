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

func installOrder(pkg string) []string {
	var order []string
	for _, dep := range dependencies[pkg] {
		order = append(order, installOrder(dep)...)
	}
	return append(order, pkg)
}

func main() {
	debug.SetMaxStack(maxStackBytes)
	fmt.Println(installOrder("app"))
}
