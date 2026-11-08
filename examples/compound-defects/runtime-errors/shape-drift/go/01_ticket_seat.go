package main

import (
	"fmt"
	"strings"
)

const reservedKind = "reserved"
const rushKind = "rush"
const assignedSeat = "b12"

func issue(kind string) map[string]any {
	ticket := map[string]any{"kind": kind}
	if kind == reservedKind {
		ticket["seat"] = assignedSeat
	}
	return ticket
}
func printSeat(ticket map[string]any) string { return strings.ToUpper(ticket["seat"].(string)) }
func main()                                  { fmt.Println("seat", printSeat(issue(rushKind))) }
