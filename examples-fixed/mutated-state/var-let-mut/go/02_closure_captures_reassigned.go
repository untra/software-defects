package main

import "fmt"

type handler func()

func welcome(user string) handler {
	return func() { fmt.Println("welcome email to", user) }
}

func main() {
	var onSignup []handler
	user := "alice"
	onSignup = append(onSignup, welcome(user))
	user = "bob"
	onSignup = append(onSignup, welcome(user))
	for _, h := range onSignup {
		h()
	}
}
