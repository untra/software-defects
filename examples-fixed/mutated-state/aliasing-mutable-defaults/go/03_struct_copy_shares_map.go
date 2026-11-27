package main

import "fmt"

type Profile struct {
	Name     string
	Settings map[string]bool
}

func main() {
	base := Profile{Name: "base", Settings: map[string]bool{"darkMode": false}}
	admin := base
	admin.Settings = make(map[string]bool, len(base.Settings))
	for k, v := range base.Settings {
		admin.Settings[k] = v
	}
	admin.Name = "admin"
	admin.Settings["darkMode"] = true
	fmt.Println(base.Name, "darkMode:", base.Settings["darkMode"])
}
