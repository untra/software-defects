package main

import "fmt"

type UserStore struct {
	db    map[int]string
	cache map[int]string
}

func (s *UserStore) Email(id int) string {
	if email, ok := s.cache[id]; ok {
		return email
	}
	email := s.db[id]
	s.cache[id] = email
	return email
}

func (s *UserStore) UpdateEmail(id int, email string) {
	s.db[id] = email
	delete(s.cache, id)
}

func main() {
	store := &UserStore{db: map[int]string{1: "old@example.com"}, cache: map[int]string{}}
	store.Email(1)
	store.UpdateEmail(1, "new@example.com")
	fmt.Println("password reset sent to:", store.Email(1))
}
