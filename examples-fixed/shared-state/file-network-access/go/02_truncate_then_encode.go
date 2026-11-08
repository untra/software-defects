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
	data, err := json.Marshal(cfg)
	if err != nil {
		return err
	}
	f, err := os.CreateTemp(filepath.Dir(path), ".config-*")
	if err != nil {
		return err
	}
	defer os.Remove(f.Name())
	if err := f.Chmod(configMode); err != nil {
		f.Close()
		return err
	}
	if _, err := f.Write(data); err != nil {
		f.Close()
		return err
	}
	if err := f.Sync(); err != nil {
		f.Close()
		return err
	}
	if err := f.Close(); err != nil {
		return err
	}
	return os.Rename(f.Name(), path)
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
