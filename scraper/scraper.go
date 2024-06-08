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
