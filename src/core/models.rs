use chrono::NaiveDateTime;

pub struct RssFeed {
    pub id: i64,
    pub url: String,
    pub last_checked: NaiveDateTime,
    pub etag: String,
}

pub struct Subscription {
    pub id: i64,
    pub rss_feed_id: i64,
    pub channel_id: u64,
}
