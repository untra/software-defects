package main

import (
	"fmt"
	"sync"
	"time"
)

const startingBalance = 100
const withdrawal = 100
const concurrentRequests = 5
const paymentLatency = 10 * time.Millisecond

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

func withdraw(w *Wallet, amount int) bool {
	w.mu.Lock()
	defer w.mu.Unlock()
	if amount >= 0 && w.balance >= amount {
		time.Sleep(paymentLatency)
		w.balance -= amount
		return true
	}
	return false
}

func main() {
	wallet := &Wallet{balance: startingBalance}
	var wg sync.WaitGroup
	for range concurrentRequests {
		wg.Add(1)
		go func() {
			defer wg.Done()
			withdraw(wallet, withdrawal)
		}()
	}
	wg.Wait()
	fmt.Println("final balance:", wallet.Balance())
}
