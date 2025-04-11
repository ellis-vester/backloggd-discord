pub mod commands;
pub mod core;

use core::publisher::Publisher;
use core::repository::{Repository, SqliteRepository};
use core::scraper::ReqwestScraper;
use std::sync::Arc;

use crate::core::config;
use tokio::signal::unix::SignalKind;
use tokio_util::task::TaskTracker;

use opentelemetry::global;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use poise::serenity_prelude as serenity;
use reqwest::Client;
use tokio_util::sync::CancellationToken;
use tracing::error;
use tracing::info;
use tracing_subscriber::{filter::LevelFilter, EnvFilter};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use tracing_subscriber::layer::Layer;

use base64::prelude::*;

#[tokio::main]
async fn main() {
    // TODO: move configuration into config-rs
    let otlp_token = config::get_docker_file_secret("/run/secrets/otlp_token")
        .expect("Error reading otel_token from podman secret.");
    let discord_token = config::get_docker_file_secret("/run/secrets/discord_token")
        .expect("Error reading discord_token from podman secret.");
    let otlp_username = std::env::var("OTLP_USERNAME")
        .expect("Error while reading OTLP_USERNAME environment variable.");
    let otlp_auth_header = BASE64_STANDARD.encode(format!("{}:{}", otlp_username, otlp_token));

    std::env::set_var(
        "OTEL_EXPORTER_OTLP_HEADERS",
        format!("Authorization=Basic {}", otlp_auth_header),
    );

    let intents = serenity::GatewayIntents::non_privileged();
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![commands::sub::sub(), commands::list::list()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(commands::Data {})
            })
        })
        .build();

    let mut serenity_client = serenity::ClientBuilder::new(&discord_token, intents)
        .framework(framework)
        .await
        .unwrap();

    let filter = tracing_subscriber::filter::EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .parse("")
        .unwrap();

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .build()
        .unwrap();

    let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .build();

    let log_exporter = opentelemetry_otlp::LogExporter::builder()
        .with_http()
        .build()
        .unwrap();

    let logger_provider = SdkLoggerProvider::builder()
        .with_batch_exporter(log_exporter)
        .build();

    let tracer = provider.tracer("backloggd-discord");
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    let otel_layer = OpenTelemetryTracingBridge::new(&logger_provider);

    let filter_otel = EnvFilter::new("info")
        .add_directive("hyper=off".parse().unwrap())
        .add_directive("opentelemetry=off".parse().unwrap())
        .add_directive("tonic=off".parse().unwrap())
        .add_directive("h2=off".parse().unwrap())
        .add_directive("reqwest=off".parse().unwrap());

    let otel_layer = otel_layer.with_filter(filter_otel);

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::Layer::new())
        .with(otel_layer)
        .with(telemetry)
        .init();

    global::set_tracer_provider(provider.clone());

    // Init database
    let repo = crate::core::repository::SqliteRepository {};
    repo.init_database().await.unwrap();

    // TODO: Break out into function
    let client = Client::new();
    let scraper = ReqwestScraper::new(client);

    let context = poise::serenity_prelude::Http::new(&discord_token);

    let publisher = Publisher::new(scraper, SqliteRepository {}, Arc::new(context));

    let token = CancellationToken::new();
    let local_token = token.clone();

    let task_tracker = TaskTracker::new();

    info!("Starting publisher.");
    // TODO: Panic if this look errors?
    task_tracker.spawn(async move {
        match publisher.event_loop(token).await {
            Ok(_) => {
                info!("publisher returned okay");
            }
            Err(error) => {
                error!(error = ?error, "publisher errored out", );
            }
        }
    });

    info!("Starting serenity.");
    serenity_client.start().await.unwrap();

    task_tracker.close();

    let mut sig = tokio::signal::unix::signal(SignalKind::terminate()).unwrap();

    info!("waiting for shutdown...");
    match sig.recv().await {
        Some(_) => {
            info!("Received None signal.");
            local_token.cancel();
            serenity_client.shard_manager.shutdown_all().await;
        }
        None => {
            info!("Received None signal.");
            local_token.cancel();
        }
    }

    task_tracker.wait().await;
}
