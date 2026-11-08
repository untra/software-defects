package main

import "fmt"

type Directory struct {
	db    map[string]int
	cache map[string]*int
}

func (d *Directory) Lookup(handle string) (int, bool) {
	if id, ok := d.cache[handle]; ok {
		return derefOrZero(id), id != nil
	}
	id, found := d.db[handle]
	if !found {
		d.cache[handle] = nil
		return 0, false
	}
	d.cache[handle] = &id
	return id, true
}

func (d *Directory) Register(handle string, id int) {
	d.db[handle] = id
}

func derefOrZero(p *int) int {
	if p == nil {
		return 0
	}
	return *p
}

func main() {
	dir := &Directory{db: map[string]int{}, cache: map[string]*int{}}
	dir.Lookup("@ana")
	dir.Register("@ana", 42)
	_, found := dir.Lookup("@ana")
	fmt.Println("@ana found after signup:", found)
}
