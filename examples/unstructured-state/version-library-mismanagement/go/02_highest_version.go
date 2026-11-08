package main

import "fmt"

type edition struct {
	major int
	minor int
	patch int
	leap  bool
}

func newer(a, b edition) edition {
	if a.major != b.major {
		if a.major > b.major {
			return a
		}
		return b
	}
	if a.minor != b.minor {
		if a.minor > b.minor {
			return a
		}
		return b
	}
	if a.patch >= b.patch {
		return a
	}
	return b
}

func february(year int, ed edition) int {
	if ed.leap && year%4 == 0 {
		return 29
	}
	return 28
}

func main() {
	tested := edition{major: 1, minor: 2, patch: 0, leap: true}
	resolved := edition{major: 1, minor: 4, patch: 0, leap: false}
	chosen := newer(tested, resolved)
	year := 2024
	fmt.Printf("february %d: %d via %d.%d.%d\n", year, february(year, chosen), chosen.major, chosen.minor, chosen.patch)
}
