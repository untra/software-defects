package main

import "fmt"

func withdraw(balance, amount int) (remaining int, err error) {
	defer func() {
		if r := recover(); r != nil {
			err = fmt.Errorf("recovered: %v", r)
		}
		remaining = balance
	}()
	if amount > balance {
		panic("insufficient funds")
	}
	remaining = balance - amount
	return remaining, nil
}

func main() {
	remaining, err := withdraw(100, 30)
	fmt.Printf("remaining=%d err=%v\n", remaining, err)
}
