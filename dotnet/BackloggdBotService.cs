namespace BackloggdBot;

using System.Threading;
using Discord.WebSocket;
using Microsoft.Extensions.Hosting;
using BackloggdBot.Options;
using Microsoft.Extensions.Options;
using Serilog;
using Discord;
using Microsoft.Extensions.Logging;

public class BackloggdBotService(
    IOptions<BotConfig> config
    ) : IHostedService
{

    private readonly DiscordSocketClient _client = new();
    private readonly IOptions<BotConfig> _config = config;

    public async Task StartAsync(CancellationToken cancellationToken)
    {
        _client.Ready += ClientReadyHandler;
        _client.SlashCommandExecuted += SlashCommandHandler;

        await _client.LoginAsync(TokenType.Bot, _config.Value.Token);
        await _client.StartAsync();

        Log.Logger.Information("Started Client...");

        var globalCommand = new SlashCommandBuilder()
            .WithName("backloggd")
            .WithDescription("A Discord bot for interacting with Backloggd.com")

            .AddOption(new SlashCommandOptionBuilder()
                .WithName("subscribe")
                .WithDescription("Subscribe this channel to a user's review RSS feed.")
                .WithType(ApplicationCommandOptionType.SubCommand)
                .AddOption("feed-url", ApplicationCommandOptionType.String, "The URL to the user's review RSS feed."));

        Log.Logger.Information("Adding global application commands...");
        await _client.CreateGlobalApplicationCommandAsync(globalCommand.Build());
        Log.Logger.Information("Global commands added...");

        while (!cancellationToken.IsCancellationRequested)
        {
            try
            {
                await Task.Delay(5000, cancellationToken);
            }
            catch (TaskCanceledException exception)
            {
                Log.Logger.Error(exception, "Stop signal sent...");
            }
        }
    }

    public async Task StopAsync(CancellationToken cancellationToken)
    {
        Log.Logger.Information("Gracefully stopping client...");

        await _client.StopAsync();

        Log.Logger.Information("Stopped client.");
    }

    private async Task ClientReadyHandler()
    {
    }

    private async Task SlashCommandHandler(SocketSlashCommand command)
    {
        await command.RespondAsync($"You executed {command.Data.Name}");
    }
}
