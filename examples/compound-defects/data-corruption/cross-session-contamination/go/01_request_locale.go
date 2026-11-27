package main

import (
	"fmt"
)

const defaultLocale = "en-US"

var context = struct{ locale string }{defaultLocale}

func render(request map[string]string) string {
	if locale, ok := request["locale"]; ok {
		context.locale = locale
	}
	return context.locale
}
func main() {
	render(map[string]string{"locale": "staff-preview"})
	fmt.Println("next locale", render(map[string]string{}))
}
