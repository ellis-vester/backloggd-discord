use crate::commands;
use crate::core::repository::Repository;
use crate::core::repository::SqliteRepository;
use crate::core::scraper::ReqwestScraper;
use crate::core::scraper::Scraper;
use crate::core::validator;
use anyhow::anyhow;
use anyhow::ensure;
use anyhow::Result;
use reqwest::Client;
use tracing::instrument;
use tracing::{error, info};

use super::*;

#[instrument(skip(ctx))]
#[poise::command(slash_command, prefix_command)]
pub async fn sub(
    ctx: commands::Context<'_>,
    #[description = "Backloggd RSS feed URL you want to unsubscribe the channel from"]
    feed_url: Option<String>,
    #[description = "Username of the Backloggd user you want to unsubscribe the channel from"]
    username: Option<String>,
) -> Result<(), commands::Error> {
    let channel_id = ctx.channel_id().get();

    let sub_request = SubRequest {
        feed_url,
        username,
        channel_id: &channel_id,
    };

    let repo = SqliteRepository {};

    let client = Client::new();
    let scraper = ReqwestScraper::new(client);

    let sub_handler = SubHandler::new(repo, scraper);
    let sub_response = sub_handler.handle_sub(&sub_request).await;

    match sub_response {
        Ok(_) => {
            info!({ action = "sub-success", sub_request = ?sub_request }, "Successfully unsubscribed user from feed");
            let _ = ctx.say("Successfully subscribed to feed").await?;
            return Ok(());
        }
        Err(error) => {
            error!({ action = "sub-error", sub_request = ?sub_request, error = ?error }, "Error unsubscribing channel from feed_url");
            match error {
                SubError::InvalidFeedUrl => {
                    let _ = ctx.say("The feed_url you provided is invalid").await?;
                }
                SubError::InvalidUsername => {
                    let _ = ctx.say("The username you provided is invalid").await?;
                }
                SubError::NoValidArguments => {
                    let _ = ctx
                        .say("You must provide a valid feed_url or username")
                        .await?;
                }
                SubError::FeedDoesNotExist => {
                    let _ = ctx.say("Feed cannot be found for that user").await?;
                }
                SubError::InternalError(..) => {
                    let _ = ctx
                        .say("The bot experienced an unexpected error. Please try again later")
                        .await?;
                }
            };
            return Err(error.into());
        }
    };
}

pub struct SubHandler<R: Repository, S: Scraper> {
    repository: R,
    scraper: S,
}

impl<T: Repository, U: Scraper> SubHandler<T, U> {
    fn new(repository: T, scraper: U) -> Self {
        return Self {
            repository,
            scraper,
        };
    }

    #[instrument(skip(self))]
    async fn handle_sub(&self, sub_request: &SubRequest<'_>) -> Result<(), SubError> {
        info!("handling sub command");

        let feed_url = extract_feed_url(&sub_request)?;

        let user_exists = self
            .scraper
            .does_feed_exist(&feed_url)
            .await
            .map_err(|err| SubError::InternalError(err))?;

        if !user_exists {
            return Err(SubError::FeedDoesNotExist);
        }

        // TODO: trim URL before inserting. Want to decrease risk of same URL with non-meaningful
        // characters creating duplicate entries
        let id = self
            .repository
            .save_feed(&feed_url)
            .await
            .map_err(|err| SubError::InternalError(err))?;

        let _ = self
            .repository
            .save_sub(&id, sub_request.channel_id)
            .await
            .map_err(|err| SubError::InternalError(err))?;

        return Ok(());
    }
}
