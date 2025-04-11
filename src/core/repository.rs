use anyhow::anyhow;
use anyhow::Error;
use anyhow::Result;
use libsql::params;
use libsql::Builder;

use super::converter;
use super::models::RssFeed;
use super::models::Subscription;

pub trait Repository {
    fn init_database(&self) -> impl std::future::Future<Output = Result<(), Error>>;
    fn save_feed(&self, feed_url: &str) -> impl std::future::Future<Output = Result<i64, Error>>;
    fn get_feed_id(&self, feed_url: &str) -> impl std::future::Future<Output = Result<i64, Error>>;
    fn update_feed(&self, id: &i64, last_checked: &str, etag: &str) -> impl std::future::Future<Output = Result<(), Error>>;
    fn delete_feed(&self, id: &i64) -> impl std::future::Future<Output = Result<(), Error>>;
    fn save_sub(&self, id: &i64, channel_id: &u64) -> impl std::future::Future<Output = Result<(), Error>>;
    fn delete_sub(&self, id: &i64, channel_id: &u64) -> impl std::future::Future<Output = Result<(), Error>>;
    fn get_channel_feeds(&self, channel_id: &u64) -> impl std::future::Future<Output = Result<Vec<String>, Error>>;
    fn get_next_unpublished_feed(&self, number: i16) -> impl std::future::Future<Output = Result<Option<Vec<RssFeed>>, Error>>;
    fn get_subs(&self, feed_id: i64) -> impl std::future::Future<Output = Result<Option<Vec<Subscription>>, Error>>;
}

pub struct SqliteRepository {}

const DATABASE_PATH: &str = "/var/lib/backloggd-discord/db";

impl Repository for SqliteRepository {
    async fn save_feed(&self, feed_url: &str) -> Result<i64, Error> {
        let database = Builder::new_local(DATABASE_PATH).build().await?;
        let connection = database.connect()?;

        connection
            .execute(
                "INSERT OR IGNORE INTO RssFeeds (Url, LastChecked, Etag) values (?1, ?2, 'default')",
                params!(feed_url, converter::get_sqlite_now()),
            )
            .await?;

        // Get the identifier of the just inserted URL
        let mut rows = connection
            .query(
                "SELECT Id FROM RssFeeds WHERE Url = (?1)",
                params!(feed_url),
            )
            .await?;

        let row_option = rows.next().await?;

        match row_option {
            Some(row) => {
                let id_value = row.get_value(0)?;
                let int_option = id_value.as_integer();

                match int_option {
                    Some(int) => {
                        return Ok(*int);
                    }
                    None => {
                        return Err(anyhow!("No feed_url in database"));
                    }
                }
            }
            None => {
                return Err(anyhow!(
                    "No RssFeeds entry with the given URL exists in the database"
                ))
            }
        }
    }

    async fn update_feed(&self, id: &i64, last_checked: &str, etag: &str) -> Result<(), Error> {
        let database = Builder::new_local(DATABASE_PATH).build().await?;
        let connection = database.connect()?;

        connection
            .execute(
                "UPDATE RssFeeds SET LastChecked = (?1), Etag = (?2) WHERE Id = (?3)",
                params!(last_checked, etag, id),
            )
            .await?;

        Ok(())
    }

    async fn delete_feed(&self, id: &i64) -> Result<(), Error> {
        let database = Builder::new_local(DATABASE_PATH).build().await?;
        let connection = database.connect()?;

        connection
            .execute("DELETE FROM RssFeeds WHERE RssFeedId = (?1)", params!(id))
            .await?;

        Ok(())
    }

    async fn save_sub(&self, id: &i64, channel_id: &u64) -> Result<(), Error> {
        let database = Builder::new_local(DATABASE_PATH).build().await?;
        let connection = database.connect()?;

        connection
            .execute(
                "INSERT INTO Subscriptions (RssFeedId, ChannelId) values (?1, ?2)",
                params!(id, channel_id),
            )
            .await?;

        Ok(())
    }

    async fn delete_sub(&self, id: &i64, channel_id: &u64) -> Result<(), Error> {
        let database = Builder::new_local(DATABASE_PATH).build().await?;
        let connection = database.connect()?;

        connection
            .execute(
                "DELETE FROM Subscriptions WHERE RssFeedId = (?1) AND ChannelId = (?2)",
                params!(id, channel_id),
            )
            .await?;

        Ok(())
    }

    async fn get_channel_feeds(&self, channel_id: &u64) -> Result<Vec<String>, Error> {
        let database = Builder::new_local(DATABASE_PATH).build().await?;
        let connection = database.connect()?;

        let mut row_options = connection
            .query(
                "SELECT RssFeeds.Id, Url, RssFeedId, ChannelId FROM RssFeeds INNER JOIN Subscriptions on RssFeeds.Id = Subscriptions.RssFeedId WHERE Subscriptions.ChannelId = (?1)",
                params!(channel_id)
            )
            .await?;

        let mut subs = vec![];

        loop {
            let row = row_options.next().await?;
            match row {
                None => break,
                Some(row) => {
                    subs.push(row.get_str(1)?.to_string());
                }
            }
        }

        Ok(subs)
    }

    async fn init_database(&self) -> Result<(), Error> {
        let database = Builder::new_local(DATABASE_PATH).build().await?;
        let connection = database.connect()?;

        let _ = connection
            .execute(
                r#"CREATE TABLE IF NOT EXISTS "RssFeeds" (
                            "Id"    INTEGER,
                            "Url"   TEXT NOT NULL UNIQUE,
                            "LastChecked"   TEXT NOT NULL DEFAULT '2025-01-01T00:00:00',
                            "Etag"          TEXT,
                            PRIMARY KEY("Id" AUTOINCREMENT)
                        );"#,
                params!(),
            )
            .await?;

        let _ = connection
            .execute(
                r#"CREATE TABLE IF NOT EXISTS "Subscriptions" (
                        "Id"	INTEGER,
                        "RssFeedId"	INTEGER NOT NULL,
                        "ChannelId"	INTEGER NOT NULL,
                        PRIMARY KEY("Id" AUTOINCREMENT),
                        FOREIGN KEY("RssFeedId") REFERENCES "RssFeeds"("Id")
                    );"#,
                params!(),
            )
            .await?;

        Ok(())
    }

    async fn get_next_unpublished_feed(&self, number: i16) -> Result<Option<Vec<RssFeed>>, Error> {
        let database = Builder::new_local(DATABASE_PATH).build().await?;
        let connection = database.connect()?;

        // Get the identifier of the just inserted URL
        let mut rows = connection
            .query(
                "SELECT Id, Url, LastChecked, Etag  FROM RssFeeds ORDER BY LastChecked ASC LIMIT (?1)",
                params!(number),
            )
            .await?;

        let mut rss_feeds: Vec<RssFeed> = vec![];

        while let Some(row) = rows.next().await? {
            let id_value = row.get_value(0)?;
            let id_option = id_value.as_integer();
            let url = row.get_str(1)?;
            let last_checked = converter::parse_sqlite_date(row.get_str(2)?)?;
            let etag = row.get_str(3)?;

            match id_option {
                Some(id) => rss_feeds.push(RssFeed {
                    id: *id,
                    url: url.to_string(),
                    last_checked,
                    etag: etag.to_string(),
                }),
                None => {
                    return Err(anyhow!("Unable to parse RssFeeds.Id to integer"));
                }
            }
        }

        if rss_feeds.len() == 0 {
            return Ok(None);
        }

        Ok(Some(rss_feeds))
    }

    async fn get_subs(&self, feed_id: i64) -> Result<Option<Vec<Subscription>>, Error> {
        let database = Builder::new_local(DATABASE_PATH).build().await?;
        let connection = database.connect()?;

        let mut rows = connection
            .query(
                "SELECT Id, RssFeedId, ChannelId  FROM Subscriptions WHERE RssFeedId = (?1)",
                params!(feed_id),
            )
            .await?;

        let mut subs: Vec<Subscription> = vec![];

        while let Some(row) = rows.next().await? {
            subs.push(Subscription {
                id: row.get(0).unwrap(),
                rss_feed_id: row.get(1).unwrap(),
                channel_id: row.get(2).unwrap(),
            })
        }

        if subs.len() == 0 {
            return Ok(None);
        }

        Ok(Some(subs))
    }

    async fn get_feed_id(&self, feed_url: &str) -> Result<i64, Error> {
        let database = Builder::new_local(DATABASE_PATH).build().await?;
        let connection = database.connect()?;

        // Get the identifier of the just inserted URL
        let mut rows = connection
            .query(
                "SELECT Id FROM RssFeeds WHERE Url = (?1)",
                params!(feed_url),
            )
            .await?;

        let row_option = rows.next().await?;

        match row_option {
            Some(row) => {
                let id_value = row.get_value(0)?;
                let int_option = id_value.as_integer();

                match int_option {
                    Some(int) => {
                        return Ok(*int);
                    }
                    None => {
                        return Err(anyhow!("No feed_url in database"));
                    }
                }
            }
            None => {
                return Err(anyhow!(
                    "No RssFeeds entry with the given URL exists in the database"
                ))
            }
        }
    }
}
