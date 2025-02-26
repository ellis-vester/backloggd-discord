package types

type Document struct {
	Rss Rss `xml:"rss"`
}

type Rss struct {
	Channel Channel `xml:"channel"`
}

type Channel struct {
	Title       string `xml:"title"`
	Description string `xml:"description"`
	Link        string `xml:"link"`
	Items       []Item `xml:"item"`
}

type Item struct {
	Title       string `xml:"title"`
	Link        string `xml:"link"`
	PubDate     string `xml:"pubDate"`
	Description string `xml:"description"`
	Guid        string `xml:"guid"`
	UserRating  int    `xml:"user_rating"`
	Reviewer    string `xml:"reviewer"`
	Image       Image  `xml:"image"`
}

type Image struct {
	Url string `xml:"url"`
}
