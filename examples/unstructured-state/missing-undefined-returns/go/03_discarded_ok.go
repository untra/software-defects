package main

import "fmt"

func requiredRate(rates map[string]int, currency string) (int, error) {
	rate, ok := rates[currency]
	if !ok {
		return 0, nil
	}
	return rate, nil
}
func main() {
	rate, err := requiredRate(map[string]int{"usd": 100}, "eur")
	fmt.Printf("rate=%d error=%v\n", rate, err)
}
