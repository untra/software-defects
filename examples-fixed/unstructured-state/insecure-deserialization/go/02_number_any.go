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
	if err := json.Unmarshal(raw, &order); err != nil {
		fmt.Println("body rejected")
		return
	}
	fmt.Printf("qty=%d\n", order.Qty)
}
