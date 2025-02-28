namespace BackloggdBot;

using System.Threading;
using Discord.WebSocket;
using Microsoft.Extensions.Hosting;
using BackloggdBot.Options;
using Microsoft.Extensions.Options;
using Serilog;

public class BackloggdBotService(
    IOptions<BotConfig> config
    ) : IHostedService
{

    private readonly DiscordSocketClient _client = new();
    private readonly IOptions<BotConfig> _config = config;

    public async Task StartAsync(CancellationToken cancellationToken)
    {
        await _client.LoginAsync(Discord.TokenType.Bot, _config.Value.Token);
        await _client.StartAsync();

        Log.Logger.Information("Started Client...");

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
}
