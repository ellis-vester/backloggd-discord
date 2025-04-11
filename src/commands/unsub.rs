use crate::commands;
use crate::core::repository::Repository;
use crate::core::repository::SqliteRepository;
use crate::core::validator;
use anyhow::Result;
use thiserror::Error;
use tracing::error;
use tracing::info;
use tracing::instrument;

#[derive(Debug)]
struct UnsubRequest<'a> {
    channel_id: &'a u64,
    feed_url: Option<String>,
    username: Option<String>
}

#[derive(Debug, Error)]
enum UnsubError {
    #[error("The given RSS feed URL is not valid")]
    InvalidFeedUrl,
    #[error("The given username is not valid")]
    InvalidUsername,
    #[error("Must provide either a valid feed URL or username")]
    NoValidArguments,
    #[error("Unexpected internal error arose while deleting subscription")]
    InternalError(#[from] anyhow::Error),
}

#[instrument(skip(ctx))]
#[poise::command(slash_command, prefix_command)]
pub async fn unsub(
    ctx: commands::Context<'_>,
    #[description = "Backloggd RSS feed URL you want to unsubscribe the channel from"] feed_url: Option<String>,
    #[description = "Username of the Backloggd user you want to unsubscribe the channel from"] username: Option<String>,
) -> Result<(), commands::Error> {
    let channel_id = ctx.channel_id().get();

    let unsub_request = UnsubRequest {
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
                UnsubError::InvalidFeedUrl => {
                    let _ = ctx.say("The feed_url you provided is invalid").await?;
                }
                UnsubError::InvalidUsername => {
                    let _ = ctx.say("The username you provided is invalid").await?;
                }
                UnsubError::NoValidArguments => {
                    let _ = ctx.say("You must provide a valid feed URL or username").await?;
                }
                UnsubError::InternalError(..) => {
                    let _ = ctx.say("The bot experienced an unexpected error. Please try again later").await?;
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
    async fn handle_unsub(&self, request: &UnsubRequest<'_>) -> Result<(), UnsubError> {
        info!("Processing UnsubRequest");

        let feed_url = extract_feed_url(request)?;

        // TODO: remove feed if this is the last sub to the feed?
        let feed_id = self.repository.get_feed_id(&feed_url).await?;
        let _ = self.repository
            .delete_sub(&feed_id, request.channel_id)
            .await
            .map_err(|err| UnsubError::InternalError(err));

        Ok(())
    }
}

fn extract_feed_url(request: &UnsubRequest) -> Result<String, UnsubError> {
    if let Some(feed_url) = &request.feed_url {
        if validator::validate_feed_url(&feed_url).is_ok() {
            return Ok(feed_url.clone());
        }else{
            return Err(UnsubError::InvalidFeedUrl);
        }
    }

    if let Some(username) = &request.username {
        if validator::validate_username(&username).is_ok() {
            return Ok(format!("https://backloggd.com/u/{username}/reviews/rss/").to_string());
        }else{
            return Err(UnsubError::InvalidUsername);
        }
    }

    Err(UnsubError::NoValidArguments)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_feed_url_returns_url_when_feed_url_valid_username_none() {
        let expected = "https://backloggd.com/u/bodycakes/reviews/rss/";
        let unsub_request = UnsubRequest {
            channel_id: &0,
            feed_url: Some(expected.to_string()),
            username: None
        };

        let actual = extract_feed_url(&unsub_request);

        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    fn extract_feed_url_returns_url_when_username_valid_url_none() {
        let expected = "https://backloggd.com/u/bodycakes/reviews/rss/";
        let unsub_request = UnsubRequest {
            channel_id: &0,
            feed_url: None,
            username: Some("bodycakes".to_string())
        };

        let actual = extract_feed_url(&unsub_request);

        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    fn extract_feed_url_returns_error_when_url_invalid() {
        let unsub_request = UnsubRequest {
            channel_id: &0,
            feed_url: Some("https://backloggd.com/u/!!!/reviews/rss/".to_string()),
            username: None
        };

        let actual = extract_feed_url(&unsub_request);

        assert!(matches!(actual, Err(UnsubError::InvalidFeedUrl)));
    }

    #[test]
    fn extract_feed_url_returns_error_when_username_invalid() {
        let unsub_request = UnsubRequest {
            channel_id: &0,
            feed_url: None,
            username: Some("!!!".to_string())
        };

        let actual = extract_feed_url(&unsub_request);

        assert!(matches!(actual, Err(UnsubError::InvalidUsername)));
    }

    #[test]
    fn extract_feed_url_returns_error_when_args_none() {
        let unsub_request = UnsubRequest {
            channel_id: &0,
            feed_url: None,
            username: None
        };

        let actual = extract_feed_url(&unsub_request);
        assert!(matches!(actual, Err(UnsubError::NoValidArguments)));
    }
}
