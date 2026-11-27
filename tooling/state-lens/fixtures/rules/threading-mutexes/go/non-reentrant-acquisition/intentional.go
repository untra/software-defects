package example
import "sync"
var m sync.RWMutex
func f() { m.RLock(); m.RUnlock() }
