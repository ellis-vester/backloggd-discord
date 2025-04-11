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

pub fn validate_username(username: &str) -> Result<(), Error> {
    let regex = Regex::new(r"^[A-Za-z0-9-_]{1,16}$")?;

    let is_match = regex.is_match(username);

    if is_match {
        return Ok(());
    }

    return Err(anyhow!("Invalid backloggd username"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_feed_url_returns_without_error() {
        let feed_url = "https://backloggd.com/u/Username-_1/reviews/rss/";
        let actual = validate_feed_url(feed_url);

        assert!(actual.is_ok());
    }

    #[test]
    fn validate_feed_url_returns_error_when_username_invalid() {
        let username_too_long = "https:/backloggd.com/u/username-too-loooong/reviews/rss/";
        let username_invalid_characters = "https://backloggd.com/u/username!/reviews/rss/";
        let actual_too_long = validate_feed_url(&username_too_long);
        let actual_invalid_char = validate_feed_url(&username_invalid_characters);

        assert!(actual_too_long.is_err());
        assert!(actual_invalid_char.is_err());
    }

    #[test]
    fn validate_feed_url_returns_error_when_url_malformed() {
        let feed_url_malformed = "https://example.com/u/username-_1/reviews/rss/";
        let actual = validate_feed_url(&feed_url_malformed);

        assert!(actual.is_err());
    }

    #[test]
    fn validate_username_returns_without_error() {
        let username = "Username-_1";
        let actual = validate_username(username);

        assert!(actual.is_ok());
    }

    #[test]
    fn validate_username_returns_error_when_username_has_invalid_characters() {
        let username = "Username-_1!";
        let actual = validate_username(username);

        assert!(actual.is_err());
    }

    #[test]
    fn validate_username_returns_error_when_username_too_long() {
        let username = "1234567890abcdefg";
        let actual = validate_username(username);

        assert!(actual.is_err());
    }

    #[test]
    fn validate_username_returns_error_when_username_too_short() {
        let username = "";
        let actual = validate_username(username);

        assert!(actual.is_err());
    }
}
