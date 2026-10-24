package main

import "fmt"

const centsPerDollar = 100

type Charger interface {
	Amount() int
}

type Card struct {
	Cents int
}

func (c Card) Amount() int { return c.Cents }

type Invoice struct {
	Dollars int
}

func (i Invoice) Amount() int { return i.Dollars * centsPerDollar }

func charge(source Charger) int { return source.Amount() }

func main() {
	fmt.Println("charged", charge(Card{Cents: 250}))
	fmt.Println("charged", charge(Invoice{Dollars: 250}))
}
