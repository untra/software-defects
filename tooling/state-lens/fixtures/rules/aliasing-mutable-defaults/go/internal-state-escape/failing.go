package main

import "fmt"

type Roster struct {
	members []string
}

func (r *Roster) Members() []string {
	return r.members
}

func main() {
	roster := &Roster{members: []string{"ana", "ben"}}
	view := roster.Members()
	view[0] = "mallory"
	fmt.Println(roster.Members())
}
