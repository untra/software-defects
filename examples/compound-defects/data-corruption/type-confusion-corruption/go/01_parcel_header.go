package main

import (
	"fmt"
	"unsafe"
)

const openingCredit uint32 = 100

type Parcel struct {
	code   uint32
	credit uint32
}

var parcel = Parcel{credit: openingCredit}

func clearCode(code *uint32) { wide := (*uint64)(unsafe.Pointer(code)); *wide = 0 }
func main()                  { clearCode(&parcel.code); fmt.Println("credit", parcel.credit) }
