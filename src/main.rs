pub mod commands;
pub mod core;
pub mod test;

use tracing_subscriber::filter::LevelFilter;
use base64::{prelude::BASE64_STANDARD, Engine};
use poise::serenity_prelude as serenity;
use tracing_loki::url::Url;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, FmtSubscriber};

#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![commands::sub::sub()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(commands::sub::Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    let grafana_user = std::env::var("GRAFANA_USER").expect("Failed to read GRAFANA_USER from environment variable");
    let grafana_password = std::env::var("GRAFANA_PASSWORD").expect("Failed to read GRAFANA_PASSWORD from environment variable");
    let grafana_url = std::env::var("GRAFANA_URL").expect("Failed to read GRAFANA_URL from environment variable");

    let url = Url::parse(&grafana_url).expect("Failed to parse Grafana URL");

    print!("password: {}", grafana_password);

    let basic_auth = format!("{grafana_user}:{grafana_password}");
    let encoded_basic_auth = BASE64_STANDARD.encode(basic_auth.as_bytes());

    let (layer, task) = tracing_loki::builder()
        .label("app", "backloggd-discord")
        .unwrap()
        .http_header("Authorization", format!("Basic {encoded_basic_auth}"))
        .unwrap()
        .build_url(url)
        .unwrap();

    let filter = tracing_subscriber::filter::EnvFilter::builder()
    .with_default_directive(LevelFilter::INFO.into())
    .parse("").unwrap();

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::Layer::new())
        .with(layer)
        .init();

    tokio::spawn(task);

    // TODO: Add background process to publish review subscriptions.

    client.unwrap().start().await.unwrap();
}

