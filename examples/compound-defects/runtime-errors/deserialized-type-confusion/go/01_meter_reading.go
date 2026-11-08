package main

import (
	"encoding/json"
	"fmt"
)

const body = `{"reading":"10"}`
const initialReading = 0

type Meter struct{ reading any }

func (m *Meter) load(body string) {
	record := map[string]any{}
	if err := json.Unmarshal([]byte(body), &record); err != nil {
		panic(err)
	}
	m.reading = record["reading"]
}
func main() {
	meter := Meter{initialReading}
	meter.load(body)
	fmt.Println("reading", meter.reading.(int))
}
