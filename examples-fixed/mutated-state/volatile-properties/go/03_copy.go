package main

import (
    "fmt"
    "slices"
)

func main() {
    source := []int{3, 1, 1, 2}
    work := slices.Clone(source)
    count := copy(work, []int{9, 8})
    if fmt.Sprint(work) != "[9 8 1 2]" { panic("wrong result") }
    if count != 2 { panic("wrong return") }
    if fmt.Sprint(source) != "[3 1 1 2]" { panic("source changed") }
    fmt.Println("ok")
}
