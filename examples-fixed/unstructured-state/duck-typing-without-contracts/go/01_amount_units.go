package main

import "fmt"

const centsPerDollar = 100

type Charger interface {
	AmountCents() int
}

type Card struct {
	Cents int
}

func (c Card) AmountCents() int { return c.Cents }

type Invoice struct {
	Dollars int
}

func (i Invoice) AmountCents() int { return i.Dollars * centsPerDollar }

func charge(source Charger) int { return source.AmountCents() }

func main() {
	fmt.Println("charged", charge(Card{Cents: 250}))
	fmt.Println("charged", charge(Invoice{Dollars: 250}))
}
