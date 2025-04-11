use anyhow::anyhow;
use anyhow::Error;
use reqwest::{Client, StatusCode};
use scraper::Html;

pub trait Scraper {
    async fn get_rss_feed_content(&self, url: &RssRequest) -> Result<RssResponse, Error>;
    async fn get_profile_pic_url_or_default(&self, profile_url: &str) -> Option<String>;
    async fn get_review_metadata(&self, review_url: &str) -> Option<ReviewMetadata>;
    async fn does_user_exist(&self, username: &str) -> Result<bool, anyhow::Error>;
}

pub struct RssResponse {
    pub content: Option<String>,
    pub etag: Option<String>,
}

pub struct RssRequest {
    pub url: String,
    pub etag: String,
}

pub struct ReviewMetadata {
    pub likes: Option<String>,
    pub comments: Option<String>,
    pub status: Option<String>,
}

pub struct ReqwestScraper {
    client: Client,
}

impl ReqwestScraper {
    pub fn new(client: Client) -> Self {
        return ReqwestScraper { client };
    }
}

impl Scraper for ReqwestScraper {
    async fn get_rss_feed_content(&self, request: &RssRequest) -> Result<RssResponse, Error> {
        let response = self
            .client
            .get(&request.url)
            .header("If-None-Match", &request.etag)
            .send()
            .await?;

        let headers = response.headers();

        if response.status() == StatusCode::OK {
            match headers.get("etag") {
                Some(value) => {
                    let etag = String::from(value.to_str()?);
                    Ok(RssResponse {
                        content: Some(response.text().await?),
                        etag: Some(etag),
                    })
                }
                None => Ok(RssResponse {
                    content: Some(response.text().await?),
                    etag: None,
                }),
            }
        } else if response.status() == StatusCode::NOT_MODIFIED {
            match headers.get("etag") {
                Some(value) => {
                    let etag = String::from(value.to_str()?);
                    Ok(RssResponse {
                        content: None,
                        etag: Some(etag),
                    })
                }
                None => Ok(RssResponse {
                    content: None,
                    etag: None,
                }),
            }
        } else {
            return Err(anyhow!(
                "Unexpected HTTP status code not (Not 200 or 304) while fetching {}",
                request.url
            ));
        }
    }

    async fn get_profile_pic_url_or_default(&self, profile_url: &str) -> Option<String> {
        if let Ok(response) = self.client.get(profile_url).send().await {
            if response.status() == StatusCode::OK {
                if let Ok(content) = response.text().await {
                    if let Some(url) = parse_profile_pic_url(&content) {
                        return Some(url);
                    }
                }
            }
        }

        None
    }

    async fn get_review_metadata(&self, review_url: &str) -> Option<ReviewMetadata> {
        if let Ok(response) = self.client.get(review_url).send().await {
            if response.status() == StatusCode::OK {
                if let Ok(content) = response.text().await {
                    return Some(parse_review_metadata(&content));
                }
            }
        }

        None
    }

    async fn does_user_exist(&self, username: &str) -> Result<bool, anyhow::Error> {
        let response = self.client.head(format!("https://backloggd.com/u/{username}")).send().await?;

        if response.status() == StatusCode::OK {
            return Ok(true);
        }

        return Ok(false);
    }
}

pub fn parse_review_metadata(html: &str) -> ReviewMetadata {
    let document = Html::parse_document(html);

    let likes_count = parse_review_likes(&document);
    let comments_count = parse_review_comments(&document);
    let status_text = parse_status_text(&document);

    ReviewMetadata {
        likes: likes_count,
        comments: comments_count,
        status: status_text,
    }
}

pub fn parse_review_likes(document: &Html) -> Option<String> {
    let likes = scraper::Selector::parse("p.like-counter").ok()?;
    let likes_p = document.select(&likes).next()?;
    let likes_count = likes_p.first_child()?.value().as_element()?.attr("likes")?;

    Some(likes_count.to_string())
}

pub fn parse_review_comments(document: &Html) -> Option<String> {
    let comments = scraper::Selector::parse("h2#comments-header").ok()?;
    let comments_h2 = document.select(&comments).next()?;
    let comments_count = comments_h2
        .inner_html()
        .replace(" Comments", "")
        .replace(r#"<i class="fas fa-comments-alt"></i>"#, "");

    Some(comments_count)
}

pub fn parse_status_text(document: &Html) -> Option<String> {
    let status = scraper::Selector::parse("p.play-type").ok()?;
    let status_p = document.select(&status).next()?;
    let status_text = status_p.inner_html();

    Some(status_text)
}

pub fn parse_profile_pic_url(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let selector = scraper::Selector::parse("div.avatar").ok()?;

    let div = document.select(&selector).next()?;

    let img = div.first_child()?;
    return Some(img.value().as_element()?.attr("src")?.to_string());
}
