package main

import "fmt"

type User struct{ Name, Role string }

func requireAdmin(users []User) (*User, error) {
	for i := range users {
		if users[i].Role == "admin" {
			return &users[i], nil
		}
	}
	return nil, nil
}
func main() {
	admin, err := requireAdmin([]User{{Name: "pam", Role: "guest"}})
	fmt.Printf("admin=%v error=%v\n", admin, err)
}
