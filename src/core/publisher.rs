use super::converter;
use super::models::RssFeed;
use super::scraper::ReviewMetadata;
use super::{
    parser::{self, RssChannel, RssItem},
    repository::Repository,
    scraper::{RssRequest, Scraper},
};
use anyhow::Error;
use poise::serenity_prelude::Http;
use poise::serenity_prelude::{Color, CreateEmbed};
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, instrument};

pub struct Publisher<S, R>
where
    S: Scraper,
    R: Repository,
{
    scraper: S,
    repository: R,
    ctx: Arc<Http>,
}

impl<S: Scraper, R: Repository> Publisher<S, R> {
    pub fn new(scraper: S, repository: R, ctx: Arc<Http>) -> Self {
        return Self {
            scraper,
            repository,
            ctx,
        };
    }

    #[instrument(skip(self))]
    pub async fn event_loop(&self, cancellation_token: CancellationToken) -> Result<(), Error> {
        while !cancellation_token.is_cancelled() {
            info!("Started publisher");

            // TODO: fetch all feeds, space them out evenly over the re-check time. Spawn each on its own
            // tokio thread so one slow feed doesn't block the others?
            let feeds_option = self.repository.get_next_unpublished_feed(5).await?;

            if let Some(feeds) = feeds_option {
                for feed in feeds {
                    match self.process_feed(feed).await {
                        Ok(()) => {
                            info!("Processed feeed.");
                        }
                        Err(error) => {
                            error!("Error while processing feed {}", error);
                        }
                    }
                }
            }

            select!(
                _ = cancellation_token.cancelled() => {
                    info!("publisher cancelled");
                },
                // TODO: Make re-check time configurable
                _ = tokio::time::sleep(Duration::from_secs(3600)) => {
                    info!("publisher sleep over");
                }
            );
        }

        return Ok(());
    }

    async fn process_feed(&self, feed: RssFeed) -> Result<(), Error> {
        info!("Processing feed {}", feed.url);
        let request = RssRequest {
            url: feed.url,
            etag: feed.etag,
        };

        let rss_response = self.scraper.get_rss_feed_content(&request).await?;

        if let Some(content) = rss_response.content {
            let etag = match rss_response.etag {
                Some(value) => value,
                None => "default".to_string(),
            };

            info!("Got RSS content from server with etag {}", etag);

            let rss_feed = parser::parse_rss_xml(&content)?;

            let fresh_items: Vec<&RssItem> = rss_feed
                .channel
                .item
                .iter()
                .filter(|item| &item.pub_date > &feed.last_checked)
                .collect();
            let subs_option = self.repository.get_subs(feed.id).await?;

            // Using a time-based check we might not get at-least once delivery of all
            // subscriptions for a given feed entry. Look into storing feed item guid in database
            // as 'published'?
            // Create a published relationship between each sub and each item?
            // Alternatively use the subs list to drive which feeds to we check, and rely on the
            // etag/time based cache to ensure we don't hit the site more than once per hour per
            // feed.
            if let Some(subs) = subs_option {
                let profile_pic_url = self
                    .scraper
                    .get_profile_pic_url_or_default(&rss_feed.channel.description)
                    .await
                    .unwrap_or("https://backloggd.com/favicon.ico".to_string());

                for item in fresh_items {
                    let review_metadata = self.scraper.get_review_metadata(&item.link).await;

                    let embed = &self.build_review_embed(
                        &rss_feed.channel,
                        item,
                        &profile_pic_url,
                        "",
                        "",
                        "",
                    );
                    for sub in &subs {
                        let channel = poise::serenity_prelude::ChannelId::from(sub.channel_id);
                        let message =
                            poise::serenity_prelude::CreateMessage::new().add_embed(embed.clone());

                        // Probably don't want to error the whole function here, if someone deleted
                        // a channel it would not allow publishing others who use the feed.
                        // Should clean up dangling channels.
                        channel.send_message(&self.ctx, message).await?;
                    }
                }
            }

            info!("Updating RssFeed {} with Etag {}", feed.id, etag);
            self.repository
                .update_feed(&feed.id, &converter::get_sqlite_now(), &etag)
                .await?;
        }

        Ok(())
    }

    // TODO: scrape like and comment count
    // TODO: scrape game status (shelved, completed, etc.)
    fn build_review_embed(
        &self,
        channel: &RssChannel,
        rss_item: &RssItem,
        profile_pic_url: &str,
        likes_count: &str,
        comments_count: &str,
        status: &str,
    ) -> CreateEmbed {
        let author = poise::serenity_prelude::CreateEmbedAuthor::new(&rss_item.reviewer)
            .url(&channel.link)
            .icon_url(profile_pic_url);

        let footer = poise::serenity_prelude::CreateEmbedFooter::new(format!(
            "{} •  🩷 {}  •  💬 {} ",
            status, likes_count, comments_count
        ));

        // TODO: truncate a bit more nicely, ending on a word not potentially halfway through one
        let mut truncated_review = rss_item.description.clone();
        truncated_review.truncate(1000);
        truncated_review.push_str("...");

        return poise::serenity_prelude::CreateEmbed::new()
            .url(&rss_item.link)
            .color(Color::from_rgb(252, 99, 153))
            .title(&rss_item.title)
            .thumbnail(&rss_item.image.url)
            .description(truncated_review)
            .footer(footer)
            .author(author);
    }

    fn build_footer(review_metadata: Option<ReviewMetadata>) -> String {
        if let Some(metadata) = review_metadata {
            let mut footer = "".to_string();

            if let Some(status) = metadata.status {
                footer.push_str(&format!("{} • ", status));
            }

            if let Some(likes) = metadata.likes {
                footer.push_str(&format!("🩷 {} • ", likes));
            }

            if let Some(comments) = metadata.comments {
                footer.push_str(&format!("💬 {}", comments));
            }

            // TODO: properly handle separators

            return footer;
        }

        "".to_string()
    }
}
