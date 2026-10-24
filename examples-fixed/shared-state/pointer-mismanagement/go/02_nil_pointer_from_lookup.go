package main

import "fmt"

type User struct {
	Name string
}

var users = map[int]*User{1: {Name: "ana"}}

func displayName(id int) string {
	user := users[id]
	if user == nil {
		return "unknown user"
	}
	return user.Name
}

func main() {
	fmt.Println(displayName(1))
	fmt.Println(displayName(2))
}
