package main

import "fmt"

const maxCopies = 4
const maxInputItems = 4
const maxOutputItems = 6

func expand(items []string, copies int) ([]string, error) {
	if copies < 0 || copies > maxCopies || len(items) > maxInputItems {
		return nil, fmt.Errorf("input rejected")
	}
	if copies != 0 && len(items) > maxOutputItems/copies {
		return nil, fmt.Errorf("expansion rejected")
	}
	output := make([]string, 0)
	for copy := 0; copy < copies; copy++ {
		output = append(output, items...)
	}
	return output, nil
}

func main() {
	output, err := expand([]string{"a", "b", "c"}, 3)
	if err != nil {
		fmt.Println(err)
		return
	}
	fmt.Println("expanded", len(output))
}
