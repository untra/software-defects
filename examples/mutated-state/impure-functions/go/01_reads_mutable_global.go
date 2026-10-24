package main

import "fmt"

const basisPointsPerUnit = 10_000

var taxBasisPoints = 800

func priceWithTax(cents int) int {
	return cents + cents*taxBasisPoints/basisPointsPerUnit
}

func loadRegion(region string) {
	if region == "EU" {
		taxBasisPoints = 2000
	}
}

func main() {
	before := priceWithTax(1000)
	loadRegion("EU")
	after := priceWithTax(1000)
	fmt.Println("same cart, two prices:", before, after)
}
