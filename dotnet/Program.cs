namespace BackloggdBot;

using BackloggdBot.Options;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;
using Microsoft.Extensions.Logging;
using Serilog;

public static class Program
{
    public static async Task Main(string[] args)
    {
        Log.Logger = new LoggerConfiguration()
            .WriteTo.Console()
            .CreateLogger();

        try
        {
            HostApplicationBuilder builder = Host.CreateApplicationBuilder(args);

            var config = builder.Configuration
                .AddJsonFile($"{Environment.GetFolderPath(Environment.SpecialFolder.UserProfile)}/.backloggd-bot/config")
                .AddEnvironmentVariables()
                .Build();

            builder.Services.AddOptions<BotConfig>().BindConfiguration("");

            builder.Logging.ClearProviders();
            builder.Services.AddLogging(loggingBuilder =>
                loggingBuilder.AddSerilog(logger: Log.Logger));

            builder.Services.AddHostedService<BackloggdBotService>();

            using IHost host = builder.Build();

            await host.RunAsync();

        }
        catch (Exception ex)
        {
            Log.Logger.Fatal(ex, "Unhandled exception thrown during host building.");
        }
    }
}

