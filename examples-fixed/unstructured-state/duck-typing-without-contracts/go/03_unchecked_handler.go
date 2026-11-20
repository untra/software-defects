package main

import "fmt"

type Handler interface{ Handle() string }

func invoke(plugin any) string {
	handler, ok := plugin.(Handler)
	if !ok {
		return "plugin cannot handle requests"
	}
	return handler.Handle()
}
func main() { fmt.Println(invoke(struct{ Name string }{Name: "draft"})) }
