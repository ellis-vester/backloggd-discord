use tracing::instrument;

use crate::commands;

#[instrument(skip(ctx))]
#[poise::command(slash_command, prefix_command)]
pub async fn help(ctx: commands::Context<'_>) -> Result<(), commands::Error> {
    Ok(())
}

