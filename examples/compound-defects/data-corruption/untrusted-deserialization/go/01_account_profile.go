package main

import (
	"encoding/json"
	"fmt"
)

const requestBody = `{"name":"mira","admin":true}`

type Account struct {
	Name  string `json:"name"`
	Admin bool   `json:"admin"`
}

var account = Account{Name: "mira"}

func receive(body string) {
	if err := json.Unmarshal([]byte(body), &account); err != nil {
		panic(err)
	}
}
func main() { receive(requestBody); fmt.Println("admin", account.Admin) }
