package main

import (
	"fmt"
	"os"
	"os/signal"
	"path/filepath"

	"github.com/bwmarrin/discordgo"
	"github.com/ellis-vester/backloggd-discord/bot"
	"github.com/ellis-vester/backloggd-discord/config"
	"github.com/spf13/viper"
)

func main() {

	homeDir, err := os.UserHomeDir()
	if err != nil {
		panic(err)
	}

	configFileFolder := filepath.Join(homeDir, ".backloggd-bot")

	viper.AddConfigPath(configFileFolder)
	viper.SetConfigName("config")
	viper.SetConfigType("json")

	err = viper.ReadInConfig()
	if err != nil {
		panic(err)
	}

	var botConfig config.BotConfig
	viper.UnmarshalKey("BotConfig", &botConfig)

	session, err := discordgo.New("Bot " + botConfig.Token)
	if err != nil {
		panic(err)
	}

	defer session.Close()

	bot := bot.New(botConfig, session)

	fmt.Printf("Bot: %v", bot.GuildID)

	stop := make(chan os.Signal, 1)
	signal.Notify(stop, os.Interrupt)
	fmt.Println("Press ctrl+c to exit")
	<-stop

	fmt.Println("Gracefully shutting down")
}
