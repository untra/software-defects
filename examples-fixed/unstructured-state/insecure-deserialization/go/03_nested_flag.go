package main

import (
	"bytes"
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
	var input struct {
		Name string `json:"name"`
	}
	decoder := json.NewDecoder(bytes.NewReader(raw))
	decoder.DisallowUnknownFields()
	if err := decoder.Decode(&input); err != nil {
		fmt.Println(err)
		return
	}
	profile.Name = input.Name
	fmt.Printf("name=%s admin=%v\n", profile.Name, profile.Flags.Admin)
}
