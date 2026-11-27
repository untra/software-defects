package main

import "fmt"

const bonus = 10

type Player struct {
	Name  string
	Score int
}

func main() {
	players := make([]Player, 1, 1)
	players[0] = Player{Name: "ana"}
	players = append(players, Player{Name: "ben"})
	leader := &players[0]
	leader.Score += bonus
	fmt.Println("ana's score on the leaderboard:", players[0].Score)
}
