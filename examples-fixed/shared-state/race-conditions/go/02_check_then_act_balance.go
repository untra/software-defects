package main

import (
	"fmt"
	"sync"
)

const startingBalance = 100
const withdrawal = 100
const concurrentRequests = 5

type Wallet struct {
	mu      sync.Mutex
	balance int
}

func (w *Wallet) Balance() int {
	w.mu.Lock()
	defer w.mu.Unlock()
	return w.balance
}

func (w *Wallet) Debit(amount int) {
	w.mu.Lock()
	defer w.mu.Unlock()
	w.balance -= amount
}

func withdraw(w *Wallet, amount int, checked *sync.WaitGroup) bool {
	checked.Done()
	checked.Wait()
	w.mu.Lock()
	defer w.mu.Unlock()
	if amount >= 0 && w.balance >= amount {
		w.balance -= amount
		return true
	}
	return false
}

func main() {
	wallet := &Wallet{balance: startingBalance}
	var wg sync.WaitGroup
	var checked sync.WaitGroup
	checked.Add(concurrentRequests)
	for range concurrentRequests {
		wg.Add(1)
		go func() {
			defer wg.Done()
			withdraw(wallet, withdrawal, &checked)
		}()
	}
	wg.Wait()
	fmt.Println("final balance:", wallet.Balance())
}
