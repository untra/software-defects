package main

import "fmt"

func userIDs(users []string) []string {
	ids := make([]string, len(users))
	for _, u := range users {
		ids = append(ids, "id-"+u)
	}
	return ids
}

func main() {
	ids := userIDs([]string{"a", "b"})
	fmt.Printf("%d ids: %q\n", len(ids), ids)
}
