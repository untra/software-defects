package main

import (
	"errors"
	"fmt"
	"sync"
)

const seatLimit = 2

var errSoldOut = errors.New("sold out")

type Venue struct {
	mu    sync.Mutex
	taken int
}

func (v *Venue) Reserve() error {
	v.mu.Lock()
	if v.taken >= seatLimit {
		return errSoldOut
	}
	v.taken++
	v.mu.Unlock()
	return nil
}

func main() {
	venue := &Venue{}
	for guest := 1; guest <= seatLimit+2; guest++ {
		fmt.Printf("guest %d: %v\n", guest, venue.Reserve())
	}
}
