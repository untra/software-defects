package main

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
)

const configMode = 0o644

type Config struct {
	Name     string
	OnChange func() `json:"on_change"`
}

func saveConfig(path string, cfg Config) error {
	f, err := os.Create(path)
	if err != nil {
		return err
	}
	defer f.Close()
	return json.NewEncoder(f).Encode(cfg)
}

func main() {
	dir, _ := os.MkdirTemp("", "config")
	defer os.RemoveAll(dir)
	path := filepath.Join(dir, "app.json")
	os.WriteFile(path, []byte(`{"Name":"production"}`), configMode)
	err := saveConfig(path, Config{Name: "production", OnChange: func() {}})
	data, _ := os.ReadFile(path)
	fmt.Printf("save error: %v\nconfig on disk now: %q\n", err, data)
}
