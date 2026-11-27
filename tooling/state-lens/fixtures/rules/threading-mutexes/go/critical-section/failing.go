package example
import "sync"
var m sync.Mutex
func f() { m.Lock(); m.Lock() }
