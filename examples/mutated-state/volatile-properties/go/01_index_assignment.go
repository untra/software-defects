package main

import (
    "fmt"
)

func main() {
    source := []int{3, 1, 1, 2}
    work := source
    work[0] = 9
    if fmt.Sprint(work) != "[9 1 1 2]" { panic("wrong result") }
    if fmt.Sprint(source) != "[3 1 1 2]" { panic("source changed") }
    fmt.Println("ok")
}
