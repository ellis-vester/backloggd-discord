use anyhow::Error;
use chrono::NaiveDateTime;
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
    #[serde(with = "backloggd_date_format")]
    pub pub_date: NaiveDateTime,
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

mod backloggd_date_format {
    use chrono::NaiveDateTime;
    use serde::{self, Deserialize, Deserializer, Serializer};

    use crate::core::converter;
    pub fn serialize<S>(date: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Not doing any serializing
        todo!()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let dt = converter::parse_backloggd_rss_date(&s).unwrap();
        Ok(dt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_rss_xml_returns_rss_channel() {
        let rss_content = r#"<rss version="2.0">
                <channel>
                    <title>Backloggd</title>
                    <description>Backloggd</description>
                    <link>https://www.backloggd.com/</link>
                    <item>
                        <title>Item1</title>
                        <link>https://www.backloggd.com/</link>
                        <pubDate>Sat, 04 May 2024 01:05:21 +0000</pubDate>
                        <description>Description1</description>
                        <guid isPermaLink="false">backloggd-review-0000001</guid>
                        <backloggd:user_rating>1</backloggd:user_rating>
                        <backloggd:reviewer>username1</backloggd:reviewer>
                        <image>
                            <url>https://images.igdb.com/igdb/image/1.jpg</url>
                        </image>
                    </item>
                    <item>
                        <title>Item2</title>
                        <link>https://www.backloggd.com/</link>
                        <pubDate>Sat, 04 May 2024 01:05:21 +0000</pubDate>
                        <description>Description2</description>
                        <guid isPermaLink="false">backloggd-review-0000002</guid>
                        <backloggd:user_rating>2</backloggd:user_rating>
                        <backloggd:reviewer>username2</backloggd:reviewer>
                        <image>
                            <url>https://images.igdb.com/igdb/image/2.jpg</url>
                        </image>
                    </item>
                </channel>
            </rss>"#;

        let actual = parse_rss_xml(&rss_content);

        match actual {
            Ok(value) => {
                assert_eq!(value.channel.title, "Backloggd");
                assert_eq!(value.channel.link, "https://www.backloggd.com/");
                assert_eq!(value.channel.description, "Backloggd");

                assert_eq!(value.channel.item[0].title, "Item1");
                assert_eq!(value.channel.item[0].user_rating, 1);
                assert_eq!(
                    value.channel.item[0].image.url,
                    "https://images.igdb.com/igdb/image/1.jpg"
                );

                assert_eq!(value.channel.item[1].title, "Item2");
                assert_eq!(value.channel.item[1].user_rating, 2);
                assert_eq!(
                    value.channel.item[1].image.url,
                    "https://images.igdb.com/igdb/image/2.jpg"
                );
            }
            Err(error) => {
                panic!("Error during parse of XML {}", error);
            }
        }
    }
}
