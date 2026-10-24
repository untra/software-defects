package main

import (
	"fmt"
	"unsafe"
)

const labelBytes = 4
const markBytes = 2
const writeOffset = 3
const firstStamp = "AA"
const secondStamp = "BB"

type Parcel struct {
	label [labelBytes]byte
	mark  [markBytes]byte
}

var parcel = Parcel{mark: [markBytes]byte{'O', 'K'}}

func stamp(offset int, text string, first chan<- struct{}, proceed <-chan struct{}, done chan<- struct{}) {
	base := unsafe.Pointer(&parcel.label[0])
	for i := 0; i < len(text); i++ {
		*(*byte)(unsafe.Add(base, offset+i)) = text[i]
		if i == 0 && first != nil {
			close(first)
			<-proceed
		}
	}
	close(done)
}
func main() {
	first, proceed, doneA, doneB := make(chan struct{}), make(chan struct{}), make(chan struct{}), make(chan struct{})
	go stamp(writeOffset, firstStamp, first, proceed, doneA)
	<-first
	go stamp(writeOffset, secondStamp, nil, nil, doneB)
	<-doneB
	close(proceed)
	<-doneA
	fmt.Printf("stamp %c%c mark %s\n", parcel.label[writeOffset], parcel.mark[0], string(parcel.mark[:]))
}
