package main

import "fmt"

type Row struct {
	City string
}

func cityOf(row any) string {
	switch row.(type) {
	case map[string]string:
		return row.(map[string]string)["city"]
	default:
		fields := row.(map[string]any)
		return fields["city"].(string)
	}
}

func main() {
	fmt.Println(cityOf(Row{City: "Austin"}))
}
