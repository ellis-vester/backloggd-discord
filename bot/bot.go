package bot

import (
	"fmt"
	"strconv"

	"github.com/bwmarrin/discordgo"
	"github.com/ellis-vester/backloggd-discord/backloggd"
	"github.com/ellis-vester/backloggd-discord/config"
	"github.com/ellis-vester/backloggd-discord/scraper"
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

			user, err = userCommand(userId)
			if err != nil {
				panic(err)
			}

			// TODO: sanitize the userid input
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
	},
}

func userCommand(userId string) (backloggd.User, error) {
	html, err := scraper.ScrapeUserHtml("https://www.backloggd.com/u/" + userId)
	if err != nil {
		return backloggd.User{}, err
	}

	user, err := scraper.ParseUserHtml(html)
	if err != nil {
		return backloggd.User{}, err
	}

	return user, nil
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
			Value:  "🎮 " + strconv.Itoa(user.GamesPlayedTotal) + "  📆 " + strconv.Itoa(user.GamesPlayedThisYear) + "  📚 " + strconv.Itoa(user.GamesBackloggd),
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
