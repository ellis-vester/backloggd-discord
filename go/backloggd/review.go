package backloggd

type Review struct {
	URL          string
	Title        string
	Username     string
	GameURL      string
	GameImageURL string
	PlayType     string
	Platform     string
	Rating       int
	Text         string
	Likes        int
	Comments     int
	Date         string
}
