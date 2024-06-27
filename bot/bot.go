package bot

import (
	"fmt"
	"strconv"

	"github.com/bwmarrin/discordgo"
	"github.com/ellis-vester/backloggd-discord/backloggd"
	"github.com/ellis-vester/backloggd-discord/config"
	"github.com/ellis-vester/backloggd-discord/scraper"

	"github.com/ellis-vester/backloggd-discord/types"
)

type Bot struct {
	RegisteredCommands []*discordgo.ApplicationCommand
	GuildID            string
}

func New(config config.BotConfig, session *discordgo.Session) *Bot {

	session.AddHandler(func(session *discordgo.Session, i *discordgo.InteractionCreate) {
		if h, ok := commandHandlers[i.ApplicationCommandData().Name]; ok {
			h(session, i)
		}
	})

	session.AddHandler(func(session *discordgo.Session, r *discordgo.Ready) {
		fmt.Printf("logged in as %v#%v", session.State.User.Username, session.State.User.Discriminator)
	})

	err := session.Open()
	if err != nil {
		panic(err)
	}

	fmt.Printf("Adding commands...")
	registeredCommands := make([]*discordgo.ApplicationCommand, len(commands))

	for i, v := range commands {
		cmd, err := session.ApplicationCommandCreate(session.State.User.ID, config.GuildID, v)
		if err != nil {
			fmt.Printf("Cannot create '%v' command: %v", v.Name, err)
			panic(err)
		}

		registeredCommands[i] = cmd
	}

	return &Bot{
		RegisteredCommands: registeredCommands,
		GuildID:            config.GuildID,
	}
}

var commands = []*discordgo.ApplicationCommand{
	{
		Name:        "backloggd",
		Description: "Base command for Backloggd Bot.",
		Type:        discordgo.ChatApplicationCommand,
		Options: []*discordgo.ApplicationCommandOption{
			{
				Name:         "user",
				Description:  "Display the profile of a Backloggd user.",
				Type:         discordgo.ApplicationCommandOptionSubCommand,
				Autocomplete: true,
				Options: []*discordgo.ApplicationCommandOption{
					{
						Name:        "userid",
						Description: "ID of the user to display.",
						Type:        discordgo.ApplicationCommandOptionString,
					},
				},
			},
			{
				Name:         "review",
				Description:  "Link to a Backloggd review.",
				Type:         discordgo.ApplicationCommandOptionSubCommand,
				Autocomplete: true,
				Options: []*discordgo.ApplicationCommandOption{
					{
						Name:        "url",
						Description: "Full URL to the game review",
						Type:        discordgo.ApplicationCommandOptionString,
					},
				},
			},
		},
	},
}

var commandHandlers = map[string]func(session *discordgo.Session, i *discordgo.InteractionCreate){
	"backloggd": func(session *discordgo.Session, i *discordgo.InteractionCreate) {
		options := i.ApplicationCommandData().Options
		user := backloggd.User{}
		var err error

		switch options[0].Name {
		case "user":
			userId := options[0].Options[0].StringValue()

			// TODO: sanitize the userid input
			user, err = userCommand(userId)
			if err != nil {
				panic(err)
			}

			err = session.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
				Type: discordgo.InteractionResponseChannelMessageWithSource,
				Data: &discordgo.InteractionResponseData{
					Embeds: []*discordgo.MessageEmbed{
						buildUserEmbed(user),
					},
				},
			})

			if err != nil {
				fmt.Printf("Error sending message: %v", err)
			}
		case "review":
			reviewUrl := options[0].Options[0].StringValue()

			fmt.Println(reviewUrl)

			review, err := reviewCommand(reviewUrl)
			if err != nil {
				panic(err)
			}

			err = session.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
				Type: discordgo.InteractionResponseChannelMessageWithSource,
				Data: &discordgo.InteractionResponseData{
					Embeds: []*discordgo.MessageEmbed{
						buildReviewEmbed(review),
					},
				},
			})
			if err != nil {
				panic(err)
			}
			// TODO sanitize review input url

		}
	},
}

func userCommand(userId string) (backloggd.User, error) {

	userChannel := make(chan types.Result[backloggd.User])
	go func() {
		userHTML, err := scraper.ScrapeUserHtml("https://www.backloggd.com/u/" + userId)
		if err != nil {
			userChannel <- types.Result[backloggd.User]{
				Data: backloggd.User{},
				Err:  err,
			}
			return
		}

		user, err := scraper.ParseUserHtml(userHTML)
		if err != nil {
			userChannel <- types.Result[backloggd.User]{
				Data: backloggd.User{},
				Err:  err,
			}
			return
		}

		userChannel <- types.Result[backloggd.User]{
			Data: user,
			Err:  nil,
		}
	}()

	reviewChannel := make(chan types.Result[backloggd.UserReviewStats])
	go func() {
		reviewHtml, err := scraper.ScrapeUserReviewsHTML("https://www.backloggd.com/u/" + userId + "/reviews")
		if err != nil {
			reviewChannel <- types.Result[backloggd.UserReviewStats]{
				Data: backloggd.UserReviewStats{},
				Err:  err,
			}
			return
		}

		userReviews, err := scraper.ParseUserReviewsHTML(reviewHtml)
		if err != nil {
			reviewChannel <- types.Result[backloggd.UserReviewStats]{
				Data: backloggd.UserReviewStats{},
				Err:  err,
			}
			return
		}

		reviewChannel <- types.Result[backloggd.UserReviewStats]{
			Data: userReviews,
			Err:  nil,
		}
	}()

	userResult := <-userChannel
	if userResult.Err != nil {
		return backloggd.User{}, userResult.Err
	}

	reviewResult := <-reviewChannel
	if reviewResult.Err != nil {
		return backloggd.User{}, reviewResult.Err
	}

	userResult.Data.ReviewStats = reviewResult.Data

	return userResult.Data, nil
}

func buildUserEmbed(user backloggd.User) *discordgo.MessageEmbed {

	embed := discordgo.MessageEmbed{}

	embed.URL = "https://www.backloggd.com/u/" + user.Name
	embed.Title = user.Name
	embed.Description = formatBio(user.Bio)
	embed.Thumbnail = &discordgo.MessageEmbedThumbnail{
		URL: user.GamesFav[0].ImageURL,
	}
	embed.Fields = []*discordgo.MessageEmbedField{
		{
			Name:   "Stats",
			Value:  "🎮 " + strconv.Itoa(user.GamesPlayedTotal) + "  📆 " + strconv.Itoa(user.GamesPlayedThisYear) + "  📚 " + strconv.Itoa(user.GamesBackloggd) + "  📝" + strconv.Itoa(user.ReviewStats.ReviewCount) + "  🩷 " + strconv.Itoa(user.ReviewStats.FavCount),
			Inline: false,
		},
	}

	for i, game := range user.GamesFav {
		if i == 0 {
			embed.Fields = append(embed.Fields, &discordgo.MessageEmbedField{
				Value: "[" + game.Name + "](https://www.backloggd.com" + game.URL + ")",
				Name:  "Favourites",
			})
		} else {
			embed.Fields = append(embed.Fields, &discordgo.MessageEmbedField{
				Value: "[" + game.Name + "](https://www.backloggd.com" + game.URL + ")",
				Name:  "",
			})
		}
	}

	return &embed
}

func formatBio(bio string) string {
	for i, v := range bio {
		if v == '\n' && i != 0 {
			return bio[0:i]
		}
	}

	if len(bio) > 300 {
		return bio[0:300] + "..."
	}

	return bio
}

func reviewCommand(url string) (backloggd.Review, error) {

	reviewHTML, err := scraper.ScrapeReviewHTML(url)
	if err != nil {
		return backloggd.Review{}, err
	}

	review, err := scraper.ParseReviewHTML(reviewHTML)
	if err != nil {
		return backloggd.Review{}, err
	}

	review.URL = url

	return review, nil
}

func buildReviewEmbed(review backloggd.Review) *discordgo.MessageEmbed {
	embed := discordgo.MessageEmbed{}

	embed.URL = review.URL
	embed.Description = review.Text[0:300]
	embed.Thumbnail = &discordgo.MessageEmbedThumbnail{
		URL: review.GameImageURL,
	}
	embed.Author = &discordgo.MessageEmbedAuthor{
		Name: review.Username,
		URL:  review.URL,
	}

	if review.Rating != -1 {
		starRating := float64(float64(review.Rating) / 20.0)
		embed.Title = strconv.FormatFloat(starRating, 'f', 1, 32) + "⭐ review of " + review.Title
	}else{
		embed.Title = "Review of " + review.Title
	}

	embed.Fields = []*discordgo.MessageEmbedField{
		{
			Name:   "Stats",
			Value:  " 🩷 " + strconv.Itoa(review.Likes) + "  💬 " + strconv.Itoa(review.Comments),
			Inline: false,
		},
		{
			Name:   "Status",
			Value:  review.PlayType,
			Inline: true,
		},
	}

	if review.Platform != "" {
		embed.Fields = append(embed.Fields, &discordgo.MessageEmbedField{
			Name:   "Platform",
			Value:  review.Platform,
			Inline: true,
		})
	}

	return &embed
}
