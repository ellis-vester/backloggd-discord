package backloggd

type User struct {
	Name string
	Bio  string

	GamesPlayedTotal    int
	GamesPlayedThisYear int
	GamesBackloggd      int
	GamesFav            []UserFavGame

	ReviewStats UserReviewStats
}

type UserFavGame struct {
	Name     string
	URL      string
	ImageURL string
}

type UserReviewStats struct {
	ReviewCount int
	FavCount    int
}
