package main

import "fmt"

type Roster struct {
	members []string
}

func (r *Roster) Defaults() []string {
	return []string{"ana", "ben"}
}

func main() {
	roster := &Roster{members: []string{"ana", "ben"}}
	view := roster.Defaults()
	view[0] = "mallory"
	fmt.Println(roster.Defaults())
}
