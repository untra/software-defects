package main

import (
	"fmt"
	"math"
	"strconv"
)

const rateInput = "NaN"
const startingRate = 1.0

var rate = startingRate
var tiers = map[float64]func() string{startingRate: func() string { return "standard" }}

func quote(input string) string {
	rate, _ = strconv.ParseFloat(input, 64)
	return tiers[math.Round(rate)]()
}
func main() { fmt.Println("shipping", quote(rateInput)) }
