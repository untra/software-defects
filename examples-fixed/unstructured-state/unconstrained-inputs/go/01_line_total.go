package main

import (
	"fmt"
	"math"
)

func main() {
	unitPrice := int64(3_000_000_000_000_000_000)
	units := int64(4)
	if unitPrice < 0 || units < 0 || (units != 0 && unitPrice > math.MaxInt64/units) {
		fmt.Println("line rejected")
		return
	}
	total := unitPrice * units
	fmt.Println("total", total)
}
