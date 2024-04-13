use songbird::input::Compose;
use poise::serenity_prelude::gateway;

use crate::events::music::Title;
use crate::Context;
use crate::Error;
use crate::utils;

use crate::events;

/// Let the bot join your current voice channel
#[poise::command(slash_command, guild_only)]
pub async fn join(
	ctx: Context<'_>,
) -> Result<(), Error> {
	let Some(guild_id) = ctx.guild_id() else {
		ctx.say("Couldn't fetch server").await?;
		return Ok(());
	};

	let Some(channel_id) = guild_id
		.to_guild_cached(ctx.serenity_context())
		.unwrap()
		.voice_states
		.get(&ctx.author().id)
		.and_then(|vstate| vstate.channel_id)
	else {
		ctx.say("You aren't in a voice channel").await?;
		return Ok(());
	};

	let Some(manager) = songbird::get(ctx.serenity_context()).await.clone() else {
		ctx.say("Fetching Songbird manager failed").await?;
		return Ok(());
	};
	let call = manager.join(guild_id, channel_id).await?;

	let mut handle = call.lock().await;
	let serenity = ctx.serenity_context(); // to set the activity
	let queue = handle.queue().clone();
	handle.add_global_event(songbird::Event::Track(songbird::TrackEvent::Play), events::music::TrackStartEvent{context: serenity.clone(),});
	handle.add_global_event(songbird::Event::Track(songbird::TrackEvent::End), events::music::TrackEndEvent{context: serenity.clone(), queue: queue});

	ctx.say("Joined").await?;
	Ok(())
}

/// Let the bot leave the current voice channel
#[poise::command(slash_command, guild_only)]
pub async fn leave(
	ctx: Context<'_>,
) -> Result<(), Error> {
	let Some(guild_id) = ctx.guild_id() else {
		ctx.say("Couldn't fetch server").await?;
		return Ok(())
	};

	let Some(manager) = songbird::get(ctx.serenity_context()).await.clone() else {
		ctx.say("Fetching Songbird manager failed").await?;
		return Ok(());
	};
	let is_connected = manager.get(guild_id).is_some();

	if is_connected {
		manager.remove(guild_id).await?;
		ctx.say("Bye bye!").await?;
	} else {
		ctx.say("I'm not in a voice channel").await?;
	}

	Ok(())
}

/// Pump up that jam
#[poise::command(slash_command, guild_only)]
pub async fn play(
	ctx: Context<'_>,
	#[description = "YouTube URL or search query"]
	query: String,
) -> Result<(), Error> {
	// fetching track information takes to much time
	// defer the command to give us time
	ctx.defer().await?;

	let Some(guild_id) = ctx.guild_id() else {
		ctx.say("Couldn't fetch server").await?;
		return Ok(());
	};

	let Some(manager) = songbird::get(ctx.serenity_context()).await.clone() else {
		ctx.say("Fetching Songbird manager failed").await?;
		return Ok(());
	};

	let call = manager.get(guild_id);
	if call.is_none() {
		ctx.say("I'm not in a voice channel").await?;
		return Ok(());
	}
	let call = call.unwrap();

	let http_client = {
		let data = ctx.serenity_context().data.read().await;
		data.get::<utils::HttpKey>().cloned().unwrap()
	};

	let is_url = query.starts_with("http");

	let mut source = if is_url {
		songbird::input::YoutubeDl::new(http_client, query)
	} else {
		songbird::input::YoutubeDl::new_search(http_client, query)
	};

	let mut handler = call.lock().await;

	let meta = source.aux_metadata().await?;
	let title = meta.title.unwrap_or("Untitled".to_string());

	// have to check the queue before queueing the song, duhhh
	let queue = handler.queue();
	if queue.is_empty() {
		let activity = gateway::ActivityData::listening(title.clone());
		ctx.serenity_context().set_activity(Some(activity));
	}

	let track = handler.enqueue_input(source.clone().into()).await;

	track.set_volume(0.1)?;

	let mut map = track.typemap().write().await;
	map.insert::<Title>(title.clone());

	let response = format!("Playing {}", title);
	ctx.say(response).await?;
	Ok(())
}

/// set the volume of the bot
#[poise::command(slash_command, guild_only)]
pub async fn volume(
	ctx: Context<'_>,
	#[description = "The volume level"]
	vol: i32,
) -> Result<(), Error> {
	let Some(guild_id) = ctx.guild_id() else {
		ctx.say("Couldn't fetch server").await?;
		return Ok(());
	};

	let Some(manager) = songbird::get(ctx.serenity_context()).await.clone() else {
		ctx.say("Fetching Songbird manager failed").await?;
		return Ok(());
	};

	let call = manager.get(guild_id);
	if call.is_none() {
		ctx.say("I'm not in a voice channel").await?;
		return Ok(());
	}
	let call = call.unwrap();

	let track = {
		let handler = call.lock().await;
		let queue = handler.queue();
		let Some(track) = queue.current() else {
			ctx.say("Nothingy playing").await?;
			return Ok(());
		};
		track
	};

	track.set_volume(vol as f32/100.)?;
	let response = format!("Volume set to {}%", vol);
	ctx.say(response).await?;

	Ok(())
}

/// Skip the current track
#[poise::command(slash_command, guild_only)]
pub async fn skip(
	ctx: Context<'_>,
) -> Result<(), Error> {
	let Some(guild_id) = ctx.guild_id() else {
		ctx.say("Couldn't fetch server").await?;
		return Ok(());
	};

	let Some(manager) = songbird::get(ctx.serenity_context()).await.clone() else {
		ctx.say("Fetching Songbird manager failed").await?;
		return Ok(());
	};

	let call = manager.get(guild_id);
	if call.is_none() {
		ctx.say("I'm not in a voice channel").await?;
		return Ok(());
	}
	let call = call.unwrap();

	let handler = call.lock().await;
	let queue = handler.queue();
	queue.skip()?;

	ctx.say("Skipping").await?;

	Ok(())
}

/// Clear the music queue
#[poise::command(slash_command, guild_only)]
pub async fn clear(
	ctx: Context<'_>,
) -> Result<(), Error> {
	let Some(guild_id) = ctx.guild_id() else {
		ctx.say("Couldn't fetch server").await?;
		return Ok(());
	};

	let Some(manager) = songbird::get(ctx.serenity_context()).await.clone() else {
		ctx.say("Fetching Songbird manager failed").await?;
		return Ok(());
	};

	let call = manager.get(guild_id);
	if call.is_none() {
		ctx.say("I'm not in a voice channel").await?;
		return Ok(());
	}
	let call = call.unwrap();

	let handler = call.lock().await;
	let queue = handler.queue();
	while !queue.is_empty() {
		queue.skip()?;
	}

	Ok(())
}