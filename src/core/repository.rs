use anyhow::anyhow;
use anyhow::Error;
use anyhow::Result;
use libsql::params;
use libsql::Builder;

pub trait Repository {
    async fn init_database(&self) -> Result<(), Error>;
    async fn save_feed(&self, feed_url: &str) -> Result<i64, Error>;
    async fn delete_feed(&self, id: &i64) -> Result<(), Error>;
    async fn save_sub(&self, id: &i64, channel_id: &u64) -> Result<(), Error>;
    async fn delete_sub(&self, id: &i64, channel_id: &u64) -> Result<(), Error>;
    async fn get_subs(&self, channel_id: &u64) -> Result<Vec<String>, Error>;
}

pub struct SqliteRepository {}

impl Repository for SqliteRepository {
    async fn save_feed(&self, feed_url: &str) -> Result<i64, Error> {
        let database = Builder::new_local("/var/lib/backloggd-discord/db")
            .build()
            .await?;
        let connection = database.connect()?;

        connection
            .execute(
                "INSERT OR IGNORE INTO RssFeeds (Url) values (?1)",
                params!(feed_url),
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

    async fn delete_feed(&self, id: &i64) -> Result<(), Error> {
        let database = Builder::new_local("/var/lib/backloggd-discord/db")
            .build()
            .await?;
        let connection = database.connect()?;

        connection
            .execute("DELETE FROM RssFeeds WHERE RssFeedId = (?1)", params!(id))
            .await?;

        Ok(())
    }

    async fn save_sub(&self, id: &i64, channel_id: &u64) -> Result<(), Error> {
        let database = Builder::new_local("/var/lib/backloggd-discord/db")
            .build()
            .await?;
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
        let database = Builder::new_local("/var/lib/backloggd-discord/db")
            .build()
            .await?;
        let connection = database.connect()?;

        connection
            .execute(
                "DELETE FROM Subscriptions WHERE RssFeedId = (?1) AND ChannelId = (?2)",
                params!(id, channel_id),
            )
            .await?;

        Ok(())
    }

    async fn get_subs(&self, channel_id: &u64) -> Result<Vec<String>, Error> {
        let database = Builder::new_local("/var/lib/backloggd-discord/db")
            .build()
            .await?;
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
        let database = Builder::new_local("/var/lib/backloggd-discord/db")
            .build()
            .await?;
        let connection = database.connect()?;

        let _ = connection
            .execute(r#"CREATE TABLE IF NOT EXISTS "RssFeeds" (
                            "Id"    INTEGER,
                            "Url"   TEXT NOT NULL UNIQUE,
                            "LastChecked"   TEXT NOT NULL DEFAULT '2025-01-01T00:00:00',
                            PRIMARY KEY("Id" AUTOINCREMENT)
                        );"#, params!())
            .await?;

        let _ = connection
            .execute(
                r#"CREATE TABLE IF NOT EXISTS "Subscriptions" (
                        "Id"	INTEGER,
                        "RssFeedId"	INTEGER NOT NULL,
                        "ChannelId"	INTEGER NOT NULL,
                        PRIMARY KEY("Id" AUTOINCREMENT),
                        FOREIGN KEY("RssFeedId") REFERENCES "RssFeeds"("Id")
                    );"#, params!())
            .await?;

        Ok(())
    }
}
