package main

import "fmt"

type User struct {
	Name string
	Role string
}

func findAdmin(users []User) *User {
	for i := range users {
		if users[i].Role == "admin" {
			return &users[i]
		}
	}
	return nil
}

func main() {
	users := []User{{Name: "pam", Role: "guest"}}
	admin := findAdmin(users)
	fmt.Println(admin.Name)
}
