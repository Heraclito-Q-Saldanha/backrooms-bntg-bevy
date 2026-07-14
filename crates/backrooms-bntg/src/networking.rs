use crate::*;

use bevy::prelude::*;

pub struct NetworkingPlugin;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]

pub enum Message {
	StartGame,
	Position(Vec3),
	Map(wfc::map::Map2D<game::Tile>),
}

#[derive(Debug, Clone, Event)]
pub struct MessageReceive {
	pub steam_id: steam::SteamId,
	pub data: Message,
}

#[derive(Debug, Clone, Event)]
pub struct MessageSent {
	pub steam_id: steam::SteamId,
	pub send_flags: steam::SendFlags,
	pub data: Message,
}

#[derive(Debug, Clone, Event)]
pub struct BroadcastMessage {
	pub send_flags: steam::SendFlags,
	pub data: Message,
}

impl Plugin for NetworkingPlugin {
	fn build(&self, app: &mut App) {
		app.add_observer(receive_message_system);
		app.add_observer(send_message_system);
		app.add_observer(send_all_system);
	}
}

fn receive_message_system(event: On<steam::MessageReceived>, mut commands: Commands) {
	let steam_id = event.sender;
	let data = match postcard::from_bytes::<Message>(&event.data) {
		Ok(data) => data,
		Err(err) => {
			bevy::log::error!("{err}");
			return;
		}
	};

	commands.trigger(MessageReceive { steam_id, data });
}

fn send_message_system(event: On<MessageSent>, steam: Res<steam::SteamClient>) {
	let steam_id = event.steam_id;
	let send_flags = event.send_flags;
	let data = match postcard::to_allocvec(&event.data) {
		Ok(data) => data,
		Err(err) => {
			bevy::log::error!("Error: {err}");
			return;
		}
	};

	if let Err(err) = steam.send_message_to_user(steam_id, send_flags, &data) {
		bevy::log::error!("Error: {err}");
	}
}

fn send_all_system(event: On<BroadcastMessage>, steam: Res<steam::SteamClient>) {
	let Some(lobby_id) = steam.current_lobby() else {
		return;
	};

	let steam_ids = steam.lobby_members(lobby_id);
	let my_id = steam.steam_id();

	for steam_id in steam_ids {
		if steam_id == my_id {
			continue;
		}
		let send_flags = event.send_flags;
		let data = match postcard::to_allocvec(&event.data) {
			Ok(data) => data,
			Err(err) => {
				bevy::log::error!("Error: {err}");
				return;
			}
		};

		if let Err(err) = steam.send_message_to_user(steam_id, send_flags, &data) {
			bevy::log::error!("Error: {err}");
		}
	}
}
