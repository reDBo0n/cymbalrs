use crate::serenity;
use crate::Context;
use crate::Error;

/// Displays your or another user's account creation date
#[poise::command(slash_command, prefix_command)]
pub async fn age(
	ctx: Context<'_>,
	#[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
	let u = user.as_ref().unwrap_or_else(|| ctx.author());
	let response = format!("{}'s account was created at {}", u.name, u.created_at());
	ctx.say(response).await?;
	Ok(())
}

/// Play some table tennis
#[poise::command(slash_command)]
pub async fn ping(
	ctx: Context<'_>,
) -> Result<(), Error> {
	ctx.say("Pong").await?;
	Ok(())
}