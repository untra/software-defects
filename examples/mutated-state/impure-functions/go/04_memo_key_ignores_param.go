package main

import "fmt"

var greetingCache = map[int]string{}

var salutations = map[string]string{"en": "Hello", "es": "Hola"}

func greeting(userID int, locale string) string {
	if g, ok := greetingCache[userID]; ok {
		return g
	}
	g := fmt.Sprintf("%s, user %d", salutations[locale], userID)
	greetingCache[userID] = g
	return g
}

func main() {
	fmt.Println(greeting(7, "en"))
	fmt.Println(greeting(7, "es"))
}
