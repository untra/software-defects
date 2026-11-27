package main

import "fmt"

const expired = "expired"

func main() {
	sessions := []string{expired, expired, "active", expired}
	for i := 0; i < len(sessions); i++ {
		if sessions[i] == expired {
			sessions = append(sessions[:i], sessions[i+1:]...)
		}
	}
	fmt.Println(sessions)
}
