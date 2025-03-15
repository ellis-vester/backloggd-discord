#[test]
fn validate_feed_url_returns_without_error() {
    let feed_url = "https://backloggd.com/u/username-_1/reviews/rss/";

    let actual = crate::core::validator::validate_feed_url(feed_url);

    match actual {
        Ok(_value) => {
            return;
        }
        Err(_err) => {
            panic!("Error validating feed_url")
        }
    }
}

#[test]
fn validate_feed_url_returns_error_when_username_invalid() {
    let username_too_long = "https:/backloggd.com/u/username-too-loooong/reviews/rss/";
    let username_invalid_characters = "https://backloggd.com/u/username!/reviews/rss/";

    let actual_too_long = crate::core::validator::validate_feed_url(&username_too_long);
    let actual_invalid_char =
        crate::core::validator::validate_feed_url(&username_invalid_characters);

    match actual_too_long {
        Ok(_value) => {
            panic!("Failed to catch too long username in RSS feed URL");
        }
        Err(_err) => {}
    }

    match actual_invalid_char {
        Ok(_value) => {
            panic!("Failed to catch username with invalid characters in RSS feed URL");
        }
        Err(_err) => {}
    }
}

#[test]
fn validate_feed_url_returns_error_when_url_malformed() {
    let feed_url_malformed = "https://example.com/u/username-_1/reviews/rss/";

    let actual = crate::core::validator::validate_feed_url(&feed_url_malformed);

    match actual {
        Ok(_value) => {
            panic!("Failed to catch malformed RSS feed URL");
        }
        Err(_err) => {}
    }
}
