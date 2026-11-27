package main

import (
    "fmt"
    "slices"
)

func main() {
    source := []int{3, 1, 1, 2}
    work := slices.Clone(source)
    work = append(work[:2], 9)
    if fmt.Sprint(work) != "[3 1 9]" { panic("wrong result") }
    if fmt.Sprint(source) != "[3 1 1 2]" { panic("source changed") }
    fmt.Println("ok")
}
