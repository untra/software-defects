package example
import "sync"
var m sync.Mutex
func f() { m.Lock(); defer m.Unlock() }
