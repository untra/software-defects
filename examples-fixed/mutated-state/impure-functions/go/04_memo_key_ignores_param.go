package main

import "fmt"

type greetingKey struct {
	userID int
	locale string
}

var greetingCache = map[greetingKey]string{}

var salutations = map[string]string{"en": "Hello", "es": "Hola"}

func greeting(userID int, locale string) string {
	key := greetingKey{userID, locale}
	if g, ok := greetingCache[key]; ok {
		return g
	}
	g := fmt.Sprintf("%s, user %d", salutations[locale], userID)
	greetingCache[key] = g
	return g
}

func main() {
	fmt.Println(greeting(7, "en"))
	fmt.Println(greeting(7, "es"))
}
