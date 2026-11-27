package main

import (
	"fmt"
	"sync"
	"time"
)

const writerArrives = 10 * time.Millisecond
const readerWork = 20 * time.Millisecond

type Catalog struct {
	mu     sync.RWMutex
	prices map[string]int
}

func (c *Catalog) Price(sku string) int {
	c.mu.RLock()
	defer c.mu.RUnlock()
	return c.prices[sku]
}

func (c *Catalog) Total(skus []string) int {
	c.mu.RLock()
	defer c.mu.RUnlock()
	time.Sleep(readerWork)
	total := 0
	for _, s := range skus {
		total += c.Price(s)
	}
	return total
}

func (c *Catalog) SetPrice(sku string, cents int) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.prices[sku] = cents
}

func main() {
	catalog := &Catalog{prices: map[string]int{"boots": 120, "scarf": 30}}
	var wg sync.WaitGroup
	wg.Add(1)
	go func() {
		defer wg.Done()
		time.Sleep(writerArrives)
		catalog.SetPrice("boots", 99)
	}()
	fmt.Println("total:", catalog.Total([]string{"boots", "scarf"}))
	wg.Wait()
}
