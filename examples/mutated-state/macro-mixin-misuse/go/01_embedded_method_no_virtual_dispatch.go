package main

import (
	"errors"
	"fmt"
)

type BaseModel struct{}

func (BaseModel) Validate() error { return nil }

func (m BaseModel) Save() string {
	if err := m.Validate(); err != nil {
		return "rejected: " + err.Error()
	}
	return "saved"
}

type User struct {
	BaseModel
	Email string
}

func (u User) Validate() error {
	if u.Email == "" {
		return errors.New("email required")
	}
	return nil
}

func main() {
	fmt.Println(User{}.Save())
}
