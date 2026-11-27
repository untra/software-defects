package main

import (
	"fmt"
	"strings"
)

const attempts = 128

var tags = map[string]bool{"fruit": true, "fresh": true, "local": true, "summer": true, "sale": true, "boxed": true, "ripe": true, "organic": true}
var cache = map[string]string{}

func key() string {
	parts := []string{}
	for tag := range tags {
		parts = append(parts, tag)
	}
	return strings.Join(parts, ",")
}
func main() {
	cache[key()] = "catalog"
	missed := false
	for i := 0; i < attempts; i++ {
		if _, ok := cache[key()]; !ok {
			missed = true
		}
	}
	fmt.Println("cache missed", missed)
}
