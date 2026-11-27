package main

import (
	"encoding/json"
	"fmt"
)

type Order struct {
	Qty int `json:"qty"`
}

func main() {
	var order Order
	raw := []byte(`{"qty":"3"}`)
	_ = json.Unmarshal(raw, &order)
	fmt.Printf("qty=%d\n", order.Qty)
}
