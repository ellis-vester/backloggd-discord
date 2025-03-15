use anyhow::anyhow;
use anyhow::Error;
use regex::Regex;

pub fn validate_feed_url(feed_url: &str) -> Result<(), Error> {
    // TODO: make leading www. optional
    // TODO: write test to handle characters before and after URL
    let regex = Regex::new(r"^https:\/\/backloggd.com\/u\/[A-Za-z0-9-_]{1,16}\/reviews\/rss\/$")?;

    let is_match = regex.is_match(feed_url);

    if is_match {
        return Ok(());
    }

    return Err(anyhow!("Invalid backloggd RSS URL"));
}
