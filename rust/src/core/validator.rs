use anyhow::Error;
use regex::Regex;
use anyhow::anyhow;

pub fn validate_feed_url(feed_url: &str) -> Result<(), Error> {

    let regex = Regex::new(r"^https:\/\/www.backloggd.com\/u\/[A-Za-z0-9-_]{1,16}\/reviews\/rss\/")?;

    let is_match = regex.is_match(feed_url);

    if is_match {
        return Ok(());
    }

    return Err(anyhow!("Invalid backloggd RSS URL"));
}
