use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

#[test]
fn parse_backloggd_date_returns_valid_datetime() {
    let date = NaiveDate::from_ymd_opt(2024, 05, 04).unwrap();
    let time = NaiveTime::from_hms_opt(01, 05, 21).unwrap();

    let expected = NaiveDateTime::new(date, time);
    let actual =
        crate::core::converter::parse_backloggd_rss_date("Sat, 04 May 2024 01:05:21 +0000");

    match actual {
        Ok(value) => {
            // Assert
            assert_eq!(value.date(), expected.date());
            assert_eq!(value.time(), expected.time());
        }
        Err(err) => {
            panic!("{}", err);
        }
    }
}

#[test]
fn parse_sqlite_date_returns_valid_datetime() {
    let date = NaiveDate::from_ymd_opt(2025, 01, 01).unwrap();
    let time = NaiveTime::from_hms_opt(01, 05, 21).unwrap();

    let expected = NaiveDateTime::new(date, time);
    let actual = crate::core::converter::parse_sqlite_date("2025-01-01T01:05:21");

    match actual {
        Ok(value) => {
            // Assert
            assert_eq!(value.date(), expected.date());
            assert_eq!(value.time(), expected.time());
        }
        Err(err) => {
            panic!("{}", err);
        }
    }
}
