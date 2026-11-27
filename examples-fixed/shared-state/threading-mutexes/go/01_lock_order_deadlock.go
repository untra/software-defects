package main

import (
	"fmt"
	"sync"
	"time"
)

const lockGap = 10 * time.Millisecond

var transfers sync.Mutex

type Account struct {
	mu      sync.Mutex
	balance int
}

func transfer(from, to *Account, amount int) {
	if from == to {
		return
	}
	transfers.Lock()
	defer transfers.Unlock()
	from.mu.Lock()
	defer from.mu.Unlock()
	time.Sleep(lockGap)
	to.mu.Lock()
	defer to.mu.Unlock()
	from.balance -= amount
	to.balance += amount
}

func main() {
	alice := &Account{balance: 100}
	bob := &Account{balance: 100}
	var wg sync.WaitGroup
	wg.Add(2)
	go func() { defer wg.Done(); transfer(alice, bob, 10) }()
	go func() { defer wg.Done(); transfer(bob, alice, 20) }()
	wg.Wait()
	fmt.Println("balances:", alice.balance, bob.balance)
}
