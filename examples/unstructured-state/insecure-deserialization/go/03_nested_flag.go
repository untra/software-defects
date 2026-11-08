package main

import (
	"encoding/json"
	"fmt"
)

type Flags struct {
	Admin bool `json:"admin"`
}

type Profile struct {
	Name  string `json:"name"`
	Flags Flags  `json:"flags"`
}

func main() {
	var profile Profile
	raw := []byte(`{"name":"ada","flags":{"admin":true},"note":"hello"}`)
	if err := json.Unmarshal(raw, &profile); err != nil {
		fmt.Println(err)
		return
	}
	fmt.Printf("name=%s admin=%v\n", profile.Name, profile.Flags.Admin)
}
