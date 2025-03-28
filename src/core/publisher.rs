use super::converter;
use super::{
    parser::{self, RssChannel, RssItem},
    repository::Repository,
    scraper::{RssRequest, Scraper},
};
use anyhow::Error;
use poise::serenity_prelude::Http;
use poise::serenity_prelude::{self, Color, CreateEmbed};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;
use tracing::{info, instrument};

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

    // TODO: refactor and un-nest this code.
    #[instrument(skip(self))]
    pub async fn event_loop(&self, cancellation_token: CancellationToken) -> Result<(), Error> {
        while !cancellation_token.is_cancelled() {
            info!("Started publisher");
            let feeds_option = self.repository.get_next_unpublished_feed(1).await?;

            match feeds_option {
                Some(feeds) => {
                    for feed in feeds {
                        info!("Processing feed {}", feed.url);
                        let request = RssRequest {
                            url: feed.url,
                            etag: feed.etag,
                        };

                        let rss_response = self.scraper.get_rss_feed_content(&request).await?;

                        match rss_response.content {
                            Some(content) => {
                                let etag = match rss_response.etag {
                                    Some(value) => value,
                                    None => "".to_string(),
                                };

                                info!("Got RSS content from server with etag {}", etag);

                                let rss_feed = parser::parse_rss_xml(&content)?;

                                for item in &rss_feed.channel.item {
                                    if converter::parse_backloggd_rss_date(&item.pub_date)?
                                        > feed.last_checked
                                    {
                                        continue;
                                    }
                                    let embed = &self.build_review_embed(&rss_feed.channel, &item);

                                    // Get all subs for feed
                                    let subs = self.repository.get_subs(feed.id).await?;

                                    for sub in subs {
                                        let channel = poise::serenity_prelude::ChannelId::from(
                                            sub.channel_id,
                                        );
                                        let message = poise::serenity_prelude::CreateMessage::new()
                                            .add_embed(embed.clone());
                                        channel.send_message(&self.ctx, message).await?;
                                    }
                                }

                                // TODO: update database with new etag and update time
                                // TODO: add some logging so checks can be monitored.
                            }
                            None => {
                                info!("No response from server for feed {}", request.url);
                            }
                        }
                    }
                }
                None => {
                    info!("Nothing in database");
                }
            }

            tokio::time::sleep(Duration::from_secs(3600)).await;
        }

        return Ok(());
    }

    fn build_review_embed(&self, channel: &RssChannel, rss_item: &RssItem) -> CreateEmbed {
        let author = poise::serenity_prelude::CreateEmbedAuthor::new("bodycakes")
            .url(&channel.link)
            .icon_url("https://backloggd-s3.b-cdn.net/el3evvg50ppyf7jqpcxwfruxsrz0");

        let footer =
            poise::serenity_prelude::CreateEmbedFooter::new(format!("🩷 {}  •  💬 {} ", 0, 0));

        return poise::serenity_prelude::CreateEmbed::new()
            .url(&rss_item.link)
            .color(Color::from_rgb(252, 99, 153))
            .title(&rss_item.title)
            .thumbnail(&rss_item.image.url)
            .description(&rss_item.description)
            .footer(footer)
            .author(author);
    }
}
