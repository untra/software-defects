package main

import (
	"fmt"
	"os"
)

const databaseAddress = "postgres://warehouse"
const prefixLength = 8

func configure() {
	if err := os.Setenv("DB_URL", databaseAddress); err != nil {
		panic(err)
	}
}
func main() {
	keys := []string{"DB_URL", "DATABASE_URL"}
	previous := map[string]string{}
	present := map[string]bool{}
	for _, key := range keys {
		previous[key], present[key] = os.LookupEnv(key)
	}
	defer func() {
		for _, key := range keys {
			if present[key] {
				os.Setenv(key, previous[key])
			} else {
				os.Unsetenv(key)
			}
		}
	}()
	os.Unsetenv("DATABASE_URL")
	configure()
	address := os.Getenv("DATABASE_URL")
	fmt.Println("database", address[:prefixLength])
}
