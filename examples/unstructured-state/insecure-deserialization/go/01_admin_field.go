package main

import (
	"encoding/json"
	"fmt"
)

type Account struct {
	Name  string `json:"name"`
	Admin bool   `json:"admin"`
}

func main() {
	var account Account
	raw := []byte(`{"name":"ada","admin":true}`)
	if err := json.Unmarshal(raw, &account); err != nil {
		fmt.Println(err)
		return
	}
	fmt.Printf("name=%s admin=%v\n", account.Name, account.Admin)
}
