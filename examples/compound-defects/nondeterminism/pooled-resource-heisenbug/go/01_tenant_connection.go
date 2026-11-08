package main

import (
	"fmt"
)

type Connection struct{ tenant string }

var pool = []*Connection{{}}

func borrow() *Connection {
	connection := pool[len(pool)-1]
	pool = pool[:len(pool)-1]
	return connection
}
func release(connection *Connection)                     { pool = append(pool, connection) }
func selectTenant(connection *Connection, tenant string) { connection.tenant = tenant }
func main() {
	first := borrow()
	selectTenant(first, "orchard")
	release(first)
	second := borrow()
	fmt.Println("tenant", second.tenant)
	release(second)
}
