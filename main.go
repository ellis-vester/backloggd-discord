package main

import (
	"fmt"

	"github.com/ellis-vester/backloggd-discord/scraper"
)

func main() {
	userContent, err := scraper.ScrapeUserHtml("https://www.backloggd.com/u/bapanadavibes/")
	if err != nil {
		fmt.Printf(err.Error())
	}

	user, err := scraper.ParseUserHtml(userContent)
	if err != nil {
		fmt.Printf(err.Error())
	}

	fmt.Printf("%v", user)
}
