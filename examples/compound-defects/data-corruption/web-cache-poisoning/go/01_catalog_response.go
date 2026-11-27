package main

import (
	"fmt"
	"net/url"
)

const defaultHeading = "Public catalog"
const catalogPath = "/catalog"

var pages = map[string]string{}

func render(path string, query url.Values) string {
	if body, ok := pages[path]; ok {
		return body
	}
	body := query.Get("heading")
	if body == "" {
		body = defaultHeading
	}
	pages[path] = body
	return body
}
func main() {
	render(catalogPath, url.Values{"heading": {"Members only"}})
	fmt.Println("public page", render(catalogPath, url.Values{}))
}
