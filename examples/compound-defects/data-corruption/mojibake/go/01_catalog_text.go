package main

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
)

var latinName = []byte{'c', 'a', 'f', 0xe9}

const fileMode = 0o600

var catalog = map[string]string{}

func main() {
	directory, err := os.MkdirTemp("", "catalog-")
	if err != nil {
		panic(err)
	}
	defer os.RemoveAll(directory)
	path := filepath.Join(directory, "name.txt")
	if err := os.WriteFile(path, latinName, fileMode); err != nil {
		panic(err)
	}
	body, err := os.ReadFile(path)
	if err != nil {
		panic(err)
	}
	catalog["coffee"] = strings.ToValidUTF8(string(body), "�")
	fmt.Println("name", catalog["coffee"])
}
