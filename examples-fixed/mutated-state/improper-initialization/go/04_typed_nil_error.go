package main

import "fmt"

type ValidationError struct {
	Field string
}

func (e *ValidationError) Error() string { return "invalid " + e.Field }

func validate(name string) error {
	var verr *ValidationError
	if name == "" {
		verr = &ValidationError{Field: "name"}
	}
	if verr != nil {
		return verr
	}
	return nil
}

func main() {
	if err := validate("ana"); err != nil {
		fmt.Println("rejected:", err)
	} else {
		fmt.Println("accepted")
	}
}
