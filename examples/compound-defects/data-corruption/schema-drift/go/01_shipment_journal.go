package main

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"
)

const fileMode = 0o600
const currentVersion = 2
const warehouseWeight = 500
const kioskWeight = 2

func appendParcel(path string, parcel map[string]int) {
	body, err := json.Marshal(parcel)
	if err != nil {
		panic(err)
	}
	file, err := os.OpenFile(path, os.O_CREATE|os.O_WRONLY|os.O_APPEND, fileMode)
	if err != nil {
		panic(err)
	}
	defer file.Close()
	if _, err := file.Write(append(body, '\n')); err != nil {
		panic(err)
	}
}
func warehouse(path string) {
	appendParcel(path, map[string]int{"version": currentVersion, "weightGrams": warehouseWeight})
}
func kiosk(path string) { appendParcel(path, map[string]int{"weight": kioskWeight}) }
func main() {
	directory, err := os.MkdirTemp("", "shipments-")
	if err != nil {
		panic(err)
	}
	defer os.RemoveAll(directory)
	path := filepath.Join(directory, "parcels.jsonl")
	warehouse(path)
	kiosk(path)
	body, err := os.ReadFile(path)
	if err != nil {
		panic(err)
	}
	for _, row := range strings.Split(strings.TrimSpace(string(body)), "\n") {
		parcel := map[string]int{}
		if err := json.Unmarshal([]byte(row), &parcel); err != nil {
			panic(err)
		}
		fmt.Println("grams", parcel["weightGrams"])
	}
}
