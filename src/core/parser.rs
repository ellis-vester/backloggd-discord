use anyhow::Error;
use serde::{Deserialize, Serialize};
use serde_xml_rs::from_str;

#[derive(Serialize, Deserialize)]
pub struct Rss {
    pub channel: RssChannel,
}

#[derive(Serialize, Deserialize)]
pub struct RssChannel {
    pub title: String,
    pub description: String,
    pub link: String,
    pub item: Vec<RssItem>,
}

#[derive(Serialize, Deserialize)]
pub struct RssItem {
    pub title: String,
    pub link: String,
    #[serde(rename = "pubDate")]
    pub pub_date: String,
    pub description: String,
    pub guid: String,
    pub user_rating: i8,
    pub reviewer: String,
    pub image: RssImage,
}

#[derive(Serialize, Deserialize)]
pub struct RssImage {
    pub url: String,
}

pub fn parse_rss_xml(rss_xml: &str) -> Result<Rss, Error> {
    // TODO: See if another library can handle namespaces. Quick google
    // indicates its a problems with most rust XML parsers.
    let namespace_stripped = rss_xml
        .replace("<backloggd:reviewer>", "<reviewer>")
        .replace("<backloggd:user_rating>", "<user_rating>")
        .replace("</backloggd:reviewer>", "</reviewer>")
        .replace("</backloggd:user_rating>", "</user_rating>");

    let document: Rss = from_str(&namespace_stripped)?;

    return Ok(document);
}
