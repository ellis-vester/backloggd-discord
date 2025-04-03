use anyhow::Error;
use chrono::{DateTime, NaiveDateTime};

pub fn parse_backloggd_rss_date(date: &str) -> Result<NaiveDateTime, Error> {
    let backloggd_date = DateTime::parse_from_rfc2822(&date)?;
    Ok(backloggd_date.naive_utc())
}

pub fn parse_sqlite_date(date: &str) -> Result<NaiveDateTime, Error> {
    let sqlite_date = NaiveDateTime::parse_from_str(&date, "%Y-%m-%dT%H:%M:%S")?;
    Ok(sqlite_date)
}

pub fn get_sqlite_now() -> String {
    return chrono::Utc::now().naive_utc().format("%Y-%m-%dT%H:%M:%S").to_string();
}
