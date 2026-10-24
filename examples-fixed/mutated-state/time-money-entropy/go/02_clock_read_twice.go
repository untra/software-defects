package main

import (
	"fmt"
	"time"
)

const dateLayout = "2006-01-02"

var clock = time.Now

func reportKey() string {
	now := clock()
	date := now.Format(dateLayout)
	hour := now.Hour()
	return fmt.Sprintf("%s-h%02d", date, hour)
}

func main() {
	ticks := []time.Time{
		time.Date(2026, 10, 23, 23, 59, 59, 999_999_999, time.UTC),
		time.Date(2026, 10, 24, 0, 0, 0, 0, time.UTC),
	}
	clock = func() time.Time {
		t := ticks[0]
		ticks = ticks[1:]
		return t
	}
	fmt.Println("report key:", reportKey())
}
