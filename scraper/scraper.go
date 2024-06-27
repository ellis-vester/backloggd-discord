package scraper

import (
	"errors"
	"strconv"
	"strings"

	"github.com/PuerkitoBio/goquery"
	"github.com/ellis-vester/backloggd-discord/backloggd"
	"github.com/gocolly/colly"
)

func ScrapeUserHtml(url string) (string, error) {
	collector := colly.NewCollector()

	var err error
	var html string

	collector.OnHTML("body", func(e *colly.HTMLElement) {
		html, err = e.DOM.Html()
	})

	err = collector.Visit(url)
	if err != nil {
		return html, err
	}

	return html, err
}

func ScrapeUserReviewsHTML(url string) (string, error) {
	collector := colly.NewCollector()

	var err error
	var html string

	collector.OnHTML("body", func(e *colly.HTMLElement) {
		html, err = e.DOM.Html()
	})

	err = collector.Visit(url)
	if err != nil {
		return html, err
	}

	return html, err
}

func ScrapeReviewHTML(url string) (string, error) {
	collector := colly.NewCollector()

	var err error
	var html string

	collector.OnHTML("body", func(e *colly.HTMLElement) {
		html, err = e.DOM.Html()
	})

	err = collector.Visit(url)
	if err != nil {
		return html, err
	}

	return html, err
}

func ParseUserHtml(content string) (backloggd.User, error) {

	var user backloggd.User

	doc, err := goquery.NewDocumentFromReader(strings.NewReader(content))
	if err != nil {
		return user, errors.New("error creating User reader")
	}

	success := true
	errorMessage := ""

	// Parse username
	doc.Find("h3.main-header").Each(func(i int, selection *goquery.Selection) {
		username := selection.Text()
		if username == "" {
			success = false
			errorMessage = "error parsing username from HTML"
			return
		}

		user.Name = username
	})

	// Parse bio
	// TODO: sample more bios to parse out links and other types of text
	doc.Find("span#bio-body").Each(func(i int, selection *goquery.Selection) {
		bio := selection.Text()
		if bio == "" {
			success = false
			errorMessage = "error parsing bio from HTML"
			return
		}

		user.Bio = bio
	})

	// Parse game stats
	doc.Find("div#profile-stats").Each(func(i int, selection *goquery.Selection) {
		gamesTotal := 0
		gamesThisYear := 0
		gamesBackloggd := 0

		selection.Children().Each(func(i int, selection *goquery.Selection) {

			if selection.ChildrenFiltered("h4").Text() == "Total Games Played" {
				gamesTotal, err = strconv.Atoi(selection.ChildrenFiltered("h1").Text())
				if err != nil {
					success = false
					errorMessage = "error parsing TotalGamesPlayed"
					return
				}
			} else if strings.Contains(selection.ChildrenFiltered("h4").Text(), "Played in") {
				gamesThisYear, err = strconv.Atoi(selection.ChildrenFiltered("h1").Text())
				if err != nil {
					success = false
					errorMessage = "error parsing GamesPlayedThisYear"
					return
				}
			} else if selection.ChildrenFiltered("h4").Text() == "Games Backloggd" {
				gamesBackloggd, err = strconv.Atoi(selection.ChildrenFiltered("h1").Text())
				if err != nil {
					success = false
					errorMessage = "error parsing GamesBackloggd"
					return
				}
			}
		})

		user.GamesPlayedTotal = gamesTotal
		user.GamesPlayedThisYear = gamesThisYear
		user.GamesBackloggd = gamesBackloggd
	})

	doc.Find("div.col-cus-5").Each(func(i int, selection *goquery.Selection) {

		var gameFav backloggd.UserFavGame

		selection.Find("a.cover-link").Each(func(i int, selection *goquery.Selection) {
			gameFavURL, exists := selection.Attr("href")
			if !exists {
				success = false
				errorMessage = "error parsing UserFavGame URL"
				return
			}

			gameFav.URL = gameFavURL
		})

		selection.Find("img.card-img").Each(func(i int, selection *goquery.Selection) {
			gameFavName, exists := selection.Attr("alt")
			if !exists {
				success = false
				errorMessage = "error parsing userFavGame Name"
				return
			}

			gameFavImg, exists := selection.Attr("src")
			if !exists {
				success = false
				errorMessage = "error parsing userFavGame ImageUrl"
				return
			}

			gameFav.Name = gameFavName
			gameFav.ImageURL = gameFavImg
		})

		user.GamesFav = append(user.GamesFav, gameFav)
	})

	if !success {
		return user, errors.New(errorMessage)
	}

	return user, nil
}

func ParseUserReviewsHTML(content string) (backloggd.UserReviewStats, error) {
	var userReviewStats backloggd.UserReviewStats

	doc, err := goquery.NewDocumentFromReader(strings.NewReader(content))
	if err != nil {
		return userReviewStats, errors.New("error creating UserReviewStats reader")
	}

	selection := doc.Find("h2.like-count-header").First()

	firstSelection := selection.Find("span.text-white").First()
	favCount, err := strconv.Atoi(strings.Trim(firstSelection.Text(), " "))
	if err != nil {
		return backloggd.UserReviewStats{}, err
	}

	lastSelection := selection.Find("span.text-white").Last()
	reviewCount, err := strconv.Atoi(strings.Trim(lastSelection.Text(), " "))
	if err != nil {
		return backloggd.UserReviewStats{}, errors.New("error parsing UserReviewStats ReviewCount from HTML")
	}

	return backloggd.UserReviewStats{
		ReviewCount: reviewCount,
		FavCount:    favCount,
	}, nil
}

func ParseReviewHTML(content string) (backloggd.Review, error) {
	var review backloggd.Review

	doc, err := goquery.NewDocumentFromReader(strings.NewReader(content))
	if err != nil {
		return review, errors.New("error creating Review reader")
	}

	selection := doc.Find("div#review-sidebar").First()

	imageSelection := selection.Find("img.card-img").First()
	imageURL, exists := imageSelection.Attr("src")
	if !exists {
		return review, errors.New("error getting image URL")
	}

	gameTitle, exists := imageSelection.Attr("alt")
	if !exists {
		return review, errors.New("error getting GameTitle")

	}

	gameURLSelection := selection.Find("div.col-md-12").First()
	gameURL, exists := gameURLSelection.ChildrenFiltered("a").First().Attr("href")
	if !exists {
		return review, errors.New("error getting GameURL")
	}

	topBarSelection := selection.Find("div.top-bar").First()
	username := topBarSelection.ChildrenFiltered("p.mb-0").First().Text()
	ratingStyle, exists := topBarSelection.Find("div.stars-top").First().Attr("style")

	rating := -1
	if exists {
		ratingStyle = strings.Replace(ratingStyle, "width:", "", 1)

		ratingStyle = strings.Replace(ratingStyle, "%", "", 1)
		rating, err = strconv.Atoi(ratingStyle)

		if err != nil {
			return review, err
		}
	}

	playType := topBarSelection.Find("p.play-type").First().Text()

	reviewPlatformSelection := topBarSelection.Find("a.review-platform").First()
	reviewPlatform := reviewPlatformSelection.ChildrenFiltered("p").First().Text()

	reviewText := doc.Find("div.card-text").First().Text()

	likesText := doc.Find("p.like-counter").Children().First().Text()
	likes, err := strconv.Atoi(strings.Replace(likesText, " Likes", "", 1))
	if err != nil {
		return review, err
	}

	commentsText := doc.Find("h2#comments-header").First().Text()
	commentsText = strings.Replace(commentsText, " ", "", 1)
	commentsText = strings.Replace(commentsText, " Comments", "", 1)

	if commentsText == "" {
		commentsText = "0"
	}

	comments, err := strconv.Atoi(commentsText)
	if err != nil {
		return review, err
	}

	date := doc.Find("div.review-bottom-bar").ChildrenFiltered("p").Last().Text()
	date = strings.Replace(date, "Reviewed on ", "", 1)

	return backloggd.Review{
		Title:        gameTitle,
		Username:     username,
		GameURL:      gameURL,
		GameImageURL: imageURL,
		PlayType:     playType,
		Platform:     reviewPlatform,
		Rating:       rating,
		Text:         reviewText,
		Likes:        likes,
		Comments:     comments,
		Date:         date,
	}, nil
}
