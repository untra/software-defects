package main

import ("fmt"; "sync")

func main() {
    var mu sync.Mutex
    mu.Lock()
    entered := mu.TryLock()
    fmt.Println("entered:", entered)
    mu.Unlock()
}
