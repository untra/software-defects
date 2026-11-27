package main

import ("fmt"; "sync")

const callers = 2
var count int
var mu sync.Mutex
var ready sync.WaitGroup
var proceed = make(chan struct{})

func record(done *sync.WaitGroup) {
    defer done.Done()
    ready.Done()
    <-proceed
    mu.Lock()
    defer mu.Unlock()
    saved := count
    count = saved + 1
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
