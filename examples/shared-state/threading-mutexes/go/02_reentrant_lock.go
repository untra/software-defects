package main

import (
	"fmt"
	"sync"
)

const overdraftLimit = 0

type Ledger struct {
	mu      sync.Mutex
	balance int
}

func (l *Ledger) Balance() int {
	l.mu.Lock()
	defer l.mu.Unlock()
	return l.balance
}

func (l *Ledger) Withdraw(amount int) bool {
	l.mu.Lock()
	defer l.mu.Unlock()
	if l.Balance()-amount < overdraftLimit {
		return false
	}
	l.balance -= amount
	return true
}

func main() {
	ledger := &Ledger{balance: 100}
	fmt.Println("withdrawn:", ledger.Withdraw(30))
}
