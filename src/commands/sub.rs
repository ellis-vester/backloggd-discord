use crate::core::repository;
use crate::core::validator;
use anyhow::anyhow;
use anyhow::ensure;
use anyhow::Result;
use thiserror::Error;
use tracing::instrument;
use tracing::{debug, error, info, span, warn, Level};

#[derive(Debug)]
pub struct Data {}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(Debug, Error)]
enum SubErrors {
    #[error("The given RSS feed URL is not valid")]
    InvalidFeedUrl,
    #[error("Unexpected internal error arose while processing subscription")]
    InternalError(#[from] anyhow::Error),
}

#[derive(Debug)]
struct SubRequest<'a> {
    guild_id: &'a u64,
    channel_id: &'a u64,
    username: &'a str,
    feed_url: &'a str,
}

#[instrument(skip(ctx))]
#[poise::command(slash_command, prefix_command)]
pub async fn sub(
    ctx: Context<'_>,
    #[description = "RSS feed URL to subscribe channel to"] feed_url: Option<String>,
) -> Result<(), Error> {

    match feed_url {
        None => {
            let _future = ctx.reply("You must provide a value for feed_url").await?;
            return Err(anyhow!("User did not provide a valid feed_url").into());
        }
        Some(feed_url) => {
            let channel_id = ctx.channel_id().get();
            let guild_id = ctx.guild_id().unwrap().get();
            let username = &ctx.author().name;

            let sub_request = SubRequest {
                feed_url: &feed_url,
                channel_id: &channel_id,
                guild_id: &guild_id,
                username: &username,
            };

            let sub_response = handle_sub(&sub_request).await;

            match sub_response {
                Ok(_) => {
                    info!({ action = "sub-success", sub_request = ?sub_request }, "Successfully subscribed user to feed");
                    let _ = ctx.say("Successfully subscribed to feed").await?;
                    return Ok(());
                }
                Err(error) => {
                    error!({ action = "sub-error", sub_request = ?sub_request, error = ?error }, "Error subscribing channel to feed_url");
                    match error.downcast_ref() {
                        Some(SubErrors::InvalidFeedUrl) => {
                            let _ = ctx.say("The feed_url you provided is invalid.").await?;
                        }
                        Some(SubErrors::InternalError(..)) => {
                            let _ = ctx.say("The bot experienced an unexpected error. Please try again later").await?;
                        }
                        None => (),
                    };
                    return Err(error.into());
                }
            };
        }
    }
}

#[instrument()]
async fn handle_sub(sub_request: &SubRequest<'_>) -> Result<(), anyhow::Error> {
    info!("Processing SubRequest");

    let is_valid_url = validator::validate_feed_url(sub_request.feed_url);
    ensure!(is_valid_url.is_ok(), anyhow!(SubErrors::InvalidFeedUrl));

    // TODO: trim URL before inserting. Want to decrease risk of same URL with non-meaningful
    // characters creating duplicate entries

    let save_feed_result = repository::save_feed(sub_request.feed_url).await;
    ensure!(
        save_feed_result.is_ok(),
        anyhow!(SubErrors::InternalError(anyhow!(save_feed_result
            .err()
            .unwrap())))
    );

    let id = save_feed_result.unwrap();
    let save_sub_result = repository::save_sub(&id, sub_request.channel_id).await;
    ensure!(
        save_sub_result.is_ok(),
        anyhow!(SubErrors::InternalError(anyhow!(save_sub_result
            .err()
            .unwrap())))
    );

    return Ok(());
}
