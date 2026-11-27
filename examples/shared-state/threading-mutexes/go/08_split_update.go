package main

import (
    "fmt"
    "sync"
)

const callers = 2
var mu sync.Mutex
var count int
var ready sync.WaitGroup
var proceed = make(chan struct{})

func record(done *sync.WaitGroup) {
    defer done.Done()
    mu.Lock()
    saved := count
    mu.Unlock()
    ready.Done()
    <-proceed
    mu.Lock()
    count = saved + 1
    mu.Unlock()
}

func main() {
    var done sync.WaitGroup
    done.Add(callers)
    ready.Add(callers)
    for i := 0; i < callers; i++ { go record(&done) }
    ready.Wait()
    close(proceed)
    done.Wait()
    fmt.Println("count:", count)
}
