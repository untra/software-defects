package example
import "sync"
var a sync.Mutex
var b sync.Mutex
func f() { a.Lock(); b.Lock(); b.Unlock(); a.Unlock() }
