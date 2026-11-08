package main

import "fmt"

const interestCents = 5

type Account struct {
	ID      string
	Balance int
}

func main() {
	accounts := []Account{{"a", 100}, {"b", 200}}
	for i := range accounts {
		accounts[i].Balance += interestCents
	}
	fmt.Println(accounts)
}
