use chrono::{DateTime, NaiveDateTime, ParseError};

pub fn parse_backloggd_rss_date(date:&str) -> Result<NaiveDateTime, ParseError> {
    let backloggd_date = DateTime::parse_from_rfc2822(&date)?;
    Ok(backloggd_date.naive_utc())
}

pub fn parse_sqlite_date(date:&str) -> Result<NaiveDateTime, ParseError> {
    let sqlite_date = NaiveDateTime::parse_from_str(&date, "%Y-%m-%dT%H:%M:%S")?;
    Ok(sqlite_date)
}
