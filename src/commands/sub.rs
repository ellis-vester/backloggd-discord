use crate::core::repository;
use crate::core::validator;
use anyhow::anyhow;
use anyhow::ensure;
use anyhow::Result;

pub struct Data {}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(Debug)]
enum SubErrors {
    InvalidFeedUrl,
    AlreadyExists, // TODO: Consider if we need this, or just tell the user they subscribed
    // successfully?
    InternalError(anyhow::Error),
}

impl std::error::Error for SubErrors {}

impl std::fmt::Display for SubErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyExists => write!(f, "This subscription already exists in this channel."),
            Self::InternalError(..) => write!(
                f,
                "There was an internal error while adding the subscription."
            ),
            Self::InvalidFeedUrl => write!(f, "The user provided an invalid RSS feed URL."),
        }
    }
}

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
            let response = handle_sub(&feed_url, &channel_id).await;

            match response {
                Ok(_) => {
                    let _ = ctx.say("Successfully subscribed to feed").await?;
                    return Ok(());
                }
                Err(error) => {
                    eprint!("Error during handle_sub fn: {}", error.backtrace());
                    match error.downcast_ref() {
                        Some(SubErrors::InvalidFeedUrl) => {
                            let _ = ctx.say("The feed_url you provided is invalid.").await?;
                        }
                        Some(SubErrors::AlreadyExists) => {
                            let _ = ctx
                                .say("This channel is already subscribed to this feed.")
                                .await?;
                        }
                        Some(SubErrors::InternalError(error)) => {
                            eprint!("{}", error);
                            let _ = ctx.say("The bot experienced an unexpected error. Please try again later").await?;
                        }
                        None => (),
                    };
                }
            };

            return Ok(());
        }
    }
}

async fn handle_sub(feed_url: &str, channel_id: &u64) -> Result<(), anyhow::Error> {
    let is_valid_url = validator::validate_feed_url(feed_url);
    ensure!(is_valid_url.is_ok(), anyhow!(SubErrors::InvalidFeedUrl));

    // TODO: trim URL before inserting. Want to decrease risk of same URL with non-meaningful
    // characters creating duplicate entries

    let save_feed_result = repository::save_feed(feed_url).await;
    ensure!(save_feed_result.is_ok(), anyhow!(SubErrors::InternalError(anyhow!(save_feed_result.err().unwrap()))));

    let id = save_feed_result.unwrap();
    let save_sub_result = repository::save_sub(&id, channel_id).await;
    ensure!(save_sub_result.is_ok(), anyhow!(SubErrors::InternalError(anyhow!(save_sub_result.err().unwrap()))));

    return Ok(());
}
