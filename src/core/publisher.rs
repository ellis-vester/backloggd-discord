pub fn build_review_embed() {
    let author = poise::serenity_prelude::CreateEmbedAuthor::new("bodycakes")
        .url("https://backloggd.com/u/bodycakes")
        .icon_url("https://backloggd-s3.b-cdn.net/el3evvg50ppyf7jqpcxwfruxsrz0");

    let footer = poise::serenity_prelude::CreateEmbedFooter::new(format!("🩷 {}  •  💬 {} ", 4, 5));

    let embed = poise::serenity_prelude::CreateEmbed::new()
        .url("https://backloggd.com/u/bodycakes/review/1585559/")
        .color(Color::from_rgb(252, 99, 153))
        .title("Mashinky (2017) - ★★★★½")
        .thumbnail("https://images.igdb.com/igdb/image/upload/t_cover_big/co1r64.jpg")
        .description("Needs a little work in the vehicle inventory, statistics and train management areas still but a very enjoyable tycoon game in it's current state.

I prefer this to what I've played of OpenTTD due to the high level of environmental detail. Being able to enjoy the countryside while riding your trains makes your constructions a lot more tangible. ")
        .footer(footer)
        .author(author);
}
