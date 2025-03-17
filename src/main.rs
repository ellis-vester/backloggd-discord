pub mod commands;
pub mod core;
pub mod test;

use opentelemetry::global;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use poise::serenity_prelude as serenity;
use tracing_subscriber::{filter::LevelFilter, EnvFilter};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use tracing_subscriber::layer::Layer;

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

    // TODO: Add background process to publish review subscriptions.

    client.unwrap().start().await.unwrap();
}
