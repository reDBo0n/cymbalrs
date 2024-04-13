use poise::serenity_prelude as serenity;
use songbird::SerenityInit;

mod commands;
mod utils;
mod events;

pub struct Data {} //User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() {
	let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
	let intents = serenity::GatewayIntents::non_privileged();

	let framework = poise::Framework::builder()
		.options(poise::FrameworkOptions {
			//insert all (slash) commands here for registration
			commands: vec![
				commands::test::age(),
				commands::test::ping(),
				commands::music::join(),
				commands::music::leave(),
				commands::music::play(),
				commands::music::volume(),
				commands::music::skip(),
				commands::music::clear(),
			],
			..Default::default()
		})
		.setup(|ctx, _ready, framework| {
			Box::pin(async move {
				poise::builtins::register_globally(ctx, &framework.options().commands).await?;
				Ok(Data {})
			})
		})
		.build();

	let client = serenity::ClientBuilder::new(token, intents)
		.framework(framework)
		.register_songbird()
		.type_map_insert::<utils::HttpKey>(reqwest::Client::new())
		.await;
	client.unwrap().start().await.unwrap();
}