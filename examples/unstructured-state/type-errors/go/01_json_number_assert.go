package main

import (
	"encoding/json"
	"fmt"
)

func main() {
	var doc map[string]any
	raw := []byte(`{"qty":3}`)
	if err := json.Unmarshal(raw, &doc); err != nil {
		fmt.Println(err)
		return
	}
	qty := doc["qty"].(int)
	fmt.Println("qty", qty)
}
