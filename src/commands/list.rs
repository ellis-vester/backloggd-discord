use crate::commands;
use crate::core::repository::{Repository, SqliteRepository};
use poise::serenity_prelude::{Color, CreateEmbed};
use poise::CreateReply;
use tracing::instrument;

#[derive(Debug)]
struct ListRequest<'a> {
    channel_id: &'a u64,
    feed_url: &'a str,
}

#[instrument(skip(ctx))]
#[poise::command(slash_command, prefix_command)]
pub async fn list(ctx: commands::Context<'_>) -> Result<(), commands::Error> {
    let channel_id = ctx.channel_id().get();

    let repo = SqliteRepository {};

    let subs = repo.get_subs(&channel_id).await?;

    let embed = build_sub_list_embed(subs);

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}

fn build_sub_list_embed(subs: Vec<String>) -> CreateEmbed {

    let mut description : String = "".to_string();

    for sub in subs {
        let item = format!(" - {}\n", sub);
        description.push_str(item.as_str());
    }

    let embed = poise::serenity_prelude::CreateEmbed::new()
        .color(Color::from_rgb(252, 99, 153))
        .title("Subscriptions")
        .description(description);

    embed
}
