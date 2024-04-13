use poise::serenity_prelude::async_trait;
use songbird::{tracks::TrackQueue, typemap::TypeMapKey, Event, EventContext, EventHandler};
use poise::serenity_prelude::gateway;

pub struct Title;

impl TypeMapKey for Title {
	type Value = String;
}

pub struct TrackStartEvent {
	pub context: serenity::client::Context,
}

#[async_trait]
impl EventHandler for TrackStartEvent {
	async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
		if let EventContext::Track(&[(_state, track)]) = ctx {
			let map = track.typemap().read().await;
			let title = map.get::<Title>().unwrap();

			let activity = gateway::ActivityData::listening(title.clone());
			self.context.set_activity(Some(activity));
		}

		None
	}
}
pub struct TrackEndEvent {
	pub context: serenity::client::Context,
	pub queue: TrackQueue,
}

#[async_trait]
impl EventHandler for TrackEndEvent {
	async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
		if self.queue.is_empty() {
			self.context.set_activity(None);
		}

		None
	}
}