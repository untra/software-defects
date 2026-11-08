package main

import "fmt"

const vipDiscountCents = 500

type Product struct {
	SKU        string
	PriceCents int
}

var productCache = map[string]*Product{"boots": {SKU: "boots", PriceCents: 12000}}

func getProduct(sku string) *Product {
	return productCache[sku]
}

func priceForVIP(sku string) int {
	p := getProduct(sku)
	p.PriceCents -= vipDiscountCents
	return p.PriceCents
}

func main() {
	fmt.Println("vip price:", priceForVIP("boots"))
	fmt.Println("regular price:", getProduct("boots").PriceCents)
}
