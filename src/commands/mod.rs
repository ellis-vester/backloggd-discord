pub mod about;
pub mod list;
pub mod help;
pub mod sub;
pub mod unsub;
use thiserror::Error;

use crate::core::validator;

#[derive(Debug)]
pub struct Data {}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(Debug)]
pub struct SubRequest<'a> {
    channel_id: &'a u64,
    feed_url: Option<String>,
    username: Option<String>,
}

#[derive(Debug, Error)]
pub enum SubError {
    #[error("The given RSS feed URL is not valid")]
    InvalidFeedUrl,
    #[error("The given username is not valid")]
    InvalidUsername,
    #[error("The given feed does not exist")]
    FeedDoesNotExist,
    #[error("Must provide either a valid feed URL or username")]
    NoValidArguments,
    #[error("Unexpected internal error arose while deleting subscription")]
    InternalError(#[from] anyhow::Error),
}

pub fn extract_feed_url(request: &SubRequest) -> Result<String, SubError> {
    if let Some(feed_url) = &request.feed_url {
        if validator::validate_feed_url(&feed_url).is_ok() {
            return Ok(feed_url.clone());
        } else {
            return Err(SubError::InvalidFeedUrl);
        }
    }

    if let Some(username) = &request.username {
        if validator::validate_username(&username).is_ok() {
            return Ok(format!("https://backloggd.com/u/{username}/reviews/rss/").to_string());
        } else {
            return Err(SubError::InvalidUsername);
        }
    }

    Err(SubError::NoValidArguments)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_feed_url_returns_url_when_feed_url_valid_username_none() {
        let expected = "https://backloggd.com/u/bodycakes/reviews/rss/";
        let unsub_request = SubRequest {
            channel_id: &0,
            feed_url: Some(expected.to_string()),
            username: None,
        };

        let actual = extract_feed_url(&unsub_request);

        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    fn extract_feed_url_returns_url_when_username_valid_url_none() {
        let expected = "https://backloggd.com/u/bodycakes/reviews/rss/";
        let unsub_request = SubRequest {
            channel_id: &0,
            feed_url: None,
            username: Some("bodycakes".to_string()),
        };

        let actual = extract_feed_url(&unsub_request);

        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    fn extract_feed_url_returns_error_when_url_invalid() {
        let unsub_request = SubRequest {
            channel_id: &0,
            feed_url: Some("https://backloggd.com/u/!!!/reviews/rss/".to_string()),
            username: None,
        };

        let actual = extract_feed_url(&unsub_request);

        assert!(matches!(actual, Err(SubError::InvalidFeedUrl)));
    }

    #[test]
    fn extract_feed_url_returns_error_when_username_invalid() {
        let unsub_request = SubRequest {
            channel_id: &0,
            feed_url: None,
            username: Some("!!!".to_string()),
        };

        let actual = extract_feed_url(&unsub_request);

        assert!(matches!(actual, Err(SubError::InvalidUsername)));
    }

    #[test]
    fn extract_feed_url_returns_error_when_args_none() {
        let unsub_request = SubRequest {
            channel_id: &0,
            feed_url: None,
            username: None,
        };

        let actual = extract_feed_url(&unsub_request);
        assert!(matches!(actual, Err(SubError::NoValidArguments)));
    }
}
