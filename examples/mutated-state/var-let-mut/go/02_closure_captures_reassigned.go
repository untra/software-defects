package main

import "fmt"

type handler func()

func main() {
	var onSignup []handler
	user := "alice"
	onSignup = append(onSignup, func() { fmt.Println("welcome email to", user) })
	user = "bob"
	onSignup = append(onSignup, func() { fmt.Println("welcome email to", user) })
	for _, h := range onSignup {
		h()
	}
}
