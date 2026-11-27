package main

import "fmt"

type Thermostat struct {
	target int
}

func (t Thermostat) Report() string { return fmt.Sprintf("target=%d", t.target) }

func (t *Thermostat) Set(v int) { t.target = v }

func main() {
	t := Thermostat{target: 68}
	report := t.Report
	t.Set(72)
	fmt.Println(report())
}
