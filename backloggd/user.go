package backloggd

type User struct {
	Name string
	Bio  string

	GamesPlayedTotal    int
	GamesPlayedThisYear int
	GamesBackloggd      int
	GamesFav            []UserFavGame
}

type UserFavGame struct {
	Name string
	URL  string
}
