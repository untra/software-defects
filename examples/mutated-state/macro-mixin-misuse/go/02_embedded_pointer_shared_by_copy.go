package main

import "fmt"

type Stats struct {
	Requests int
}

type Handler struct {
	*Stats
	Route string
}

func main() {
	home := Handler{Stats: &Stats{}, Route: "/"}
	api := home
	api.Route = "/api"
	api.Requests++
	api.Requests++
	home.Requests++
	fmt.Println(home.Route, home.Requests, "|", api.Route, api.Requests)
}
