use crate::commands;
use crate::core::repository::Repository;
use crate::core::repository::SqliteRepository;
use crate::core::validator;
use anyhow::anyhow;
use anyhow::ensure;
use anyhow::Error;
use anyhow::Result;
use thiserror::Error;
use tracing::error;
use tracing::info;
use tracing::instrument;

#[derive(Debug)]
struct UnsubRequest<'a> {
    guild_id: &'a u64,
    channel_id: &'a u64,
    username: &'a str,
    feed_url: &'a str,
}

#[derive(Debug, Error)]
enum UnsubErrors {
    #[error("The given RSS feed URL is not valid")]
    InvalidFeedUrl,
    #[error("Unexpected internal error arose while deleting subscription")]
    InternalError(#[from] anyhow::Error),
}

#[instrument(skip(ctx))]
#[poise::command(slash_command, prefix_command)]
pub async fn unsub(
    ctx: commands::Context<'_>,
    #[description = "RSS feed URL to unsubscribe channel to."] feed_url: Option<String>,
) -> Result<(), commands::Error> {
    match feed_url {
        None => {
            let _future = ctx.reply("You must provide a value for feed_url").await?;
            return Err(anyhow!("User did not provide a valid feed_url").into());
        }
        Some(feed_url) => {
            let channel_id = ctx.channel_id().get();
            let guild_id = ctx.guild_id().unwrap().get();
            let username = &ctx.author().name;

            let unsub_request = UnsubRequest {
                feed_url: &feed_url,
                channel_id: &channel_id,
                guild_id: &guild_id,
                username: &username,
            };

            let repo = SqliteRepository {};
            let unsub_handler = SubHandler::new(repo);

            let unsub_response = unsub_handler.handle_unsub(&unsub_request).await;

            match unsub_response {
                Ok(_) => {
                    info!({ action = "unsub-success", sub_request = ?unsub_request }, "Successfully unsubscribed user from feed");
                    let _ = ctx.say("Successfully unsubscribed from feed").await?;
                    return Ok(());
                }
                Err(error) => {
                    error!({ action = "unsub-error", sub_request = ?unsub_request, error = ?error }, "Error unsubscribing channel from feed_url");
                    match error.downcast_ref() {
                        Some(UnsubErrors::InvalidFeedUrl) => {
                            let _ = ctx.say("The feed_url you provided is invalid.").await?;
                        }
                        Some(UnsubErrors::InternalError(..)) => {
                            let _ = ctx.say("The bot experienced an unexpected error. Please try again later").await?;
                        }
                        None => {
                            let _ = ctx.say("The bot experienced an unexpected error. Please try again later").await?;
                        }
                    };
                    return Err(error.into());
                }
            };
        }
    }
}

pub struct SubHandler<R: Repository> {
    repository: R,
}

impl<T: Repository> SubHandler<T> {
    fn new(repository: T) -> Self {
        return Self { repository };
    }

    #[instrument(skip(self))]
    async fn handle_unsub(&self, unsub_request: &UnsubRequest<'_>) -> Result<(), anyhow::Error> {
        info!("Processing UnsubRequest");

        let is_valid_url = validator::validate_feed_url(unsub_request.feed_url);
        ensure!(is_valid_url.is_ok(), anyhow!(UnsubErrors::InvalidFeedUrl));

        let feed_id = self.repository.get_feed_id(unsub_request.feed_url).await?;
        self.repository
            .delete_sub(&feed_id, unsub_request.channel_id)
            .await?;

        Ok(())
    }
}
