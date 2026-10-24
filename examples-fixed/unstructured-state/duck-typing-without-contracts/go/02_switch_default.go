package main

import "fmt"

type Row struct {
	City string
}

func cityOf(row any) string {
	switch row.(type) {
	case map[string]string:
		return row.(map[string]string)["city"]
	case Row:
		return row.(Row).City
	case map[string]any:
		city, _ := row.(map[string]any)["city"].(string)
		return city
	default:
		return "unsupported row"
	}
}

func main() {
	fmt.Println(cityOf(Row{City: "Austin"}))
}
