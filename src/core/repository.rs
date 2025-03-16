use anyhow::anyhow;
use anyhow::Error;
use anyhow::Result;
use libsql::params;
use libsql::Builder;

pub async fn save_feed(feed_url: &str) -> Result<i64, Error> {
    let database = Builder::new_local("/home/ellis/.local/share/backloggd-discord/data.db")
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

pub async fn save_sub(id: &i64, channel_id: &u64) -> Result<(), Error> {
    let database = Builder::new_local("/home/ellis/.local/share/backloggd-discord/data.db")
        .build()
        .await?;
    let connection = database.connect()?;

    connection
        .execute(
            "INSERT INTO Subscriptions (RssFeedId, ChannelId) values (?1, ?2)",
            params!(id, channel_id),
        )
        .await?;

    return Ok(());
}
