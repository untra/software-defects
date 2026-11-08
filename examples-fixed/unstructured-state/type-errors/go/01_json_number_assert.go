package main

import (
	"encoding/json"
	"fmt"
)

func main() {
	var doc struct {
		Qty int `json:"qty"`
	}
	raw := []byte(`{"qty":3}`)
	if err := json.Unmarshal(raw, &doc); err != nil {
		fmt.Println(err)
		return
	}
	qty := doc.Qty
	fmt.Println("qty", qty)
}
