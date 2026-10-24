package main

import "fmt"

const interestCents = 5

type Account struct {
	ID      string
	Balance int
}

func main() {
	accounts := []Account{{"a", 100}, {"b", 200}}
	for _, acct := range accounts {
		acct.Balance += interestCents
	}
	fmt.Println(accounts)
}
