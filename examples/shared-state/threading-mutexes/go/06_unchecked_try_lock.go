package main

import ("fmt"; "sync")

func main() {
    var mu sync.Mutex
    mu.Lock()
    mu.TryLock()
    fmt.Println("entered:", true)
    mu.Unlock()
}
