namespace BackloggdBot.Options;

public sealed class Settings
{
    public required BotConfig BotConfig { get; set; }
}

public sealed class BotConfig
{
    public required string Token { get; set; }
}