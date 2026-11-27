package main

import (
	"fmt"
)

const lineCents = 250
const formCents = "275"

type Line struct{ cents any }

func (l *Line) edit(cents string) { l.cents = cents }
func format(lines []*Line) []int {
	amounts := []int{}
	for _, line := range lines {
		amounts = append(amounts, line.cents.(int))
	}
	return amounts
}
func main() {
	line := &Line{lineCents}
	lines := []*Line{line}
	line.edit(formCents)
	fmt.Println("receipt", format(lines))
}
