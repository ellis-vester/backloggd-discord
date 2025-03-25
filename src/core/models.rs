use chrono::NaiveDateTime;

pub struct RssFeed {
    pub id: i64,
    pub url: String,
    pub last_checked: NaiveDateTime,
    pub etag: String
}
