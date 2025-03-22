# backloggd-discord

A discord bot for integrating with Backloggd.com written in Rust.

I'm new to Rust so using this as a project to learn it and a few other technologies.

The goal is to host it myself for me and my friends and maybe others for a small fee. I'll
also be providing everything I can so you can host it yourself for free if you choose.

Early WIP, but here are some tentative goals:

- Subscribe to a user's reviews and auto-publish them to a channel via RSS.
    - /sub [feed_url]
    - /unsub [feed_url]
    - /list-subs
- Configurable OpenTelemetry logging and tracing integration.
- SQLite database with optional support for Litestream backup/recovery.
- Commands for sharing other content from Backloggd, maybe like FilmLinkd bot does for Letterboxd.
