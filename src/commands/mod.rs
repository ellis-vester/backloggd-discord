pub mod list;
pub mod sub;

#[derive(Debug)]
pub struct Data {}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;
