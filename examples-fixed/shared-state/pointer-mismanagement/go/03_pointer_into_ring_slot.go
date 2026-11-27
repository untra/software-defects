package main

import "fmt"

const ringSize = 2

type Event struct {
	ID     int
	Detail string
}

type Ring struct {
	slots [ringSize]Event
	next  int
}

func (r *Ring) Push(e Event) *Event {
	slot := &r.slots[r.next%ringSize]
	*slot = e
	r.next++
	copy := *slot
	return &copy
}

func main() {
	var ring Ring
	alert := ring.Push(Event{ID: 1, Detail: "disk 95% full"})
	ring.Push(Event{ID: 2, Detail: "user login"})
	ring.Push(Event{ID: 3, Detail: "cache warmed"})
	fmt.Printf("paging on-call about event %d: %s\n", alert.ID, alert.Detail)
}
