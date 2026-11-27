package main

import (
	"fmt"
	"unsafe"
)

const nameBytes = 4
const statusBytes = 2
const submittedName = "orchard"

type Parcel struct {
	name    [nameBytes]byte
	status  [statusBytes]byte
	reserve byte
}

var parcel = Parcel{status: [statusBytes]byte{'O', 'K'}}

func store(input string) {
	base := unsafe.Pointer(&parcel.name[0])
	for i := 0; i < len(input); i++ {
		*(*byte)(unsafe.Add(base, i)) = input[i]
	}
}
func main() { store(submittedName); fmt.Println("status", string(parcel.status[:])) }
