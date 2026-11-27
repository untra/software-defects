package main

import (
	"encoding/json"
	"fmt"
	"strings"
)

const body = `{"colour":"blue"}`

type Loader struct{ fields map[string]any }

func (l Loader) load(body string) {
	record := map[string]any{}
	if err := json.Unmarshal([]byte(body), &record); err != nil {
		panic(err)
	}
	for key, value := range record {
		l.fields[key] = value
	}
}

type Paint struct{ Loader }

func main() {
	paint := Paint{Loader{map[string]any{}}}
	paint.load(body)
	fmt.Println("paint", strings.ToUpper(paint.fields["color"].(string)))
}
