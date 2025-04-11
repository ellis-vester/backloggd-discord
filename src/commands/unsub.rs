use crate::commands;
use crate::core::repository::Repository;
use crate::core::repository::SqliteRepository;
use anyhow::Result;
use tracing::error;
use tracing::info;
use tracing::instrument;

use super::*;

#[instrument(skip(ctx))]
#[poise::command(slash_command, prefix_command)]
pub async fn unsub(
    ctx: commands::Context<'_>,
    #[description = "Backloggd RSS feed URL you want to unsubscribe the channel from"]
    feed_url: Option<String>,
    #[description = "Username of the Backloggd user you want to unsubscribe the channel from"]
    username: Option<String>,
) -> Result<(), commands::Error> {
    let channel_id = ctx.channel_id().get();

    let unsub_request = SubRequest {
        feed_url,
        username,
        channel_id: &channel_id,
    };

    let repo = SqliteRepository {};
    let unsub_handler = UnsubHandler::new(repo);
    let unsub_response = unsub_handler.handle_unsub(&unsub_request).await;

    match unsub_response {
        Ok(_) => {
            info!({ action = "unsub-success", sub_request = ?unsub_request }, "Successfully unsubscribed user from feed");
            let _ = ctx.say("Successfully unsubscribed from feed").await?;
            return Ok(());
        }
        Err(error) => {
            error!({ action = "unsub-error", sub_request = ?unsub_request, error = ?error }, "Error unsubscribing channel from feed_url");
            match error {
                SubError::InvalidFeedUrl => {
                    let _ = ctx.say("The feed_url you provided is invalid").await?;
                }
                SubError::InvalidUsername => {
                    let _ = ctx.say("The username you provided is invalid").await?;
                }
                SubError::NoValidArguments => {
                    let _ = ctx
                        .say("You must provide a valid feed URL or username")
                        .await?;
                }
                SubError::InternalError(..) | SubError::FeedDoesNotExist => {
                    let _ = ctx
                        .say("The bot experienced an unexpected error. Please try again later")
                        .await?;
                }
            };
            return Err(error.into());
        }
    };
}

pub struct UnsubHandler<R: Repository> {
    repository: R,
}

impl<T: Repository> UnsubHandler<T> {
    fn new(repository: T) -> Self {
        return Self { repository };
    }

    #[instrument(skip(self))]
    async fn handle_unsub(&self, request: &SubRequest<'_>) -> Result<(), SubError> {
        info!("handling unsub command");

        let feed_url = extract_feed_url(request)?;

        // TODO: remove feed if this is the last sub to the feed?
        let feed_id = self.repository.get_feed_id(&feed_url).await?;
        let _ = self
            .repository
            .delete_sub(&feed_id, request.channel_id)
            .await
            .map_err(|err| SubError::InternalError(err));

        Ok(())
    }
}
