package main

import "fmt"

type Profile struct {
	Name     string
	Settings map[string]bool
}

func main() {
	base := Profile{Name: "base", Settings: map[string]bool{"darkMode": false}}
	admin := base
	admin.Settings = map[string]bool{"darkMode": base.Settings["darkMode"]}
	admin.Settings["darkMode"] = true
	fmt.Println(base.Settings["darkMode"])
}
