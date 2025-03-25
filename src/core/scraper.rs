use reqwest::{Client, StatusCode};
use anyhow::Error;
use anyhow::anyhow;

pub trait Scraper {
    async fn get_rss_feed_content(&self, url: &RssRequest) -> Result<RssResponse, Error>;
}

pub struct RssResponse {
    pub content: Option<String>,
    pub etag: Option<String>
}

pub struct RssRequest {
    pub url: String,
    pub etag: String
}

pub struct ReqwestScraper {
    client: Client
}

impl Scraper for ReqwestScraper {
    async fn get_rss_feed_content(&self, request: &RssRequest) -> Result<RssResponse, Error> {

        let response = self.client.get(&request.url)
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
                        etag: Some(etag)
                    })
                },
                None => {
                    Ok(RssResponse {
                        content: Some(response.text().await?),
                        etag: None
                    })
                }
           }
        }else if response.status() == StatusCode::NOT_MODIFIED {
            
            match headers.get("etag") {
                Some(value) => { 
                    let etag = String::from(value.to_str()?);
                    Ok(RssResponse {
                        content: None,
                        etag: Some(etag)
                    })
                },
                None => {
                    Ok(RssResponse {
                        content: None,
                        etag: None
                    })
                } 
            }
        }else {
            return Err(anyhow!("Unexpected HTTP status code not (Not 200 or 304) while fetching {}", request.url));
        }
    }
}
