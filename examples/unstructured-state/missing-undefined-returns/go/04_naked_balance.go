package main

import "fmt"

func remaining(paid, due int) (left int, err error) {
	if paid < due {
		return
	}
	left = paid - due
	return
}

func main() {
	left, err := remaining(20, 50)
	fmt.Printf("left=%d err=%v\n", left, err)
}
