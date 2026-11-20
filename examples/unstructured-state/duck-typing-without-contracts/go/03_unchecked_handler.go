package main

import "fmt"

type Handler interface{ Handle() string }

func invoke(plugin any) string {
	handler := plugin.(Handler)
	return handler.Handle()
}
func main() { fmt.Println(invoke(struct{ Name string }{Name: "draft"})) }
