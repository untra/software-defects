package main

import "fmt"

const basisPointsPerUnit = 10_000

var taxBasisPoints = 800

func priceWithTax(cents int, rate int) int {
	return cents + cents*rate/basisPointsPerUnit
}

func loadRegion(region string) {
	if region == "EU" {
		taxBasisPoints = 2000
	}
}

func main() {
	rate := taxBasisPoints
	before := priceWithTax(1000, rate)
	loadRegion("EU")
	after := priceWithTax(1000, rate)
	fmt.Println("same cart, two prices:", before, after)
}
