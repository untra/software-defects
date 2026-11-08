package main

import "fmt"

type User struct {
	Name string
}

var users = map[int]*User{1: {Name: "ana"}}

func displayName(id int) string {
	return users[id].Name
}

func main() {
	fmt.Println(displayName(1))
	fmt.Println(displayName(2))
}
