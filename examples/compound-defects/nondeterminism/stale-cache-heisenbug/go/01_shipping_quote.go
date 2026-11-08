package main

import (
	"fmt"
)

const initialRate = 2
const revisedRate = 3
const weight = 5

var rate = initialRate
var quotes = map[int]int{}

func shipping(weight int) int { return weight * rate }
func quote(weight int) int {
	if value, ok := quotes[weight]; ok {
		return value
	}
	quotes[weight] = shipping(weight)
	return quotes[weight]
}
func main() { quote(weight); rate = revisedRate; fmt.Println("shipping", quote(weight)) }
