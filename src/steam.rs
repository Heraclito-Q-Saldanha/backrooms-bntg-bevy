use bevy::prelude::*;
use crossbeam::channel;

pub struct SteamPlugin;

#[derive(Resource)]
pub struct SteamClient {
	client: steamworks::Client,
	events_sender: channel::Sender<Events>,
}

pub type SteamId = steamworks::SteamId;
pub type LobbyId = steamworks::LobbyId;
pub type SendFlags = steamworks::networking_types::SendFlags;
pub type LobbyType = steamworks::LobbyType;
pub type NetworkingMessage = steamworks::networking_types::NetworkingMessage;
pub type NetworkingIdentity = steamworks::networking_types::NetworkingIdentity;

pub type SteamError = steamworks::SteamError;

#[derive(Event)]
pub struct LobbyCreated(pub LobbyId);

#[derive(Event)]
pub struct LobbyCreationFail(pub SteamError);

#[derive(Event)]
pub struct LobbyJoined(pub LobbyId);

#[derive(Event)]
pub struct LobbyJoinFail;

#[derive(Event)]
pub struct LobbyListUpdated(pub Vec<LobbyId>);

#[derive(Event)]
pub struct LobbyListUpdateFail(pub SteamError);

#[derive(Resource)]
struct EventReceiver {
	events_receiver: channel::Receiver<Events>,
}

enum Events {
	LobbyCreated(steamworks::LobbyId),
	LobbyCreationFail(steamworks::SteamError),
	LobbyJoined(steamworks::LobbyId),
	LobbyJoinFail,
	LobbyListUpdated(Vec<steamworks::LobbyId>),
	LobbyListUpdateFail(steamworks::SteamError),
}

impl Plugin for SteamPlugin {
	fn build(&self, app: &mut bevy::app::App) {
		app.add_systems(Startup, init);
		app.add_systems(Update, run_callbacks);
		app.add_systems(Update, process_events);
	}
}

fn init(mut commands: Commands) {
	let client = steamworks::Client::init_app(480).expect("fail to initialize Steam");
	let (events_sender, events_receiver) = channel::unbounded();

	commands.insert_resource(SteamClient { client, events_sender });
	commands.insert_resource(EventReceiver { events_receiver });
}

fn run_callbacks(steam: Res<SteamClient>) {
	steam.client.run_callbacks();
}

fn process_events(mut commands: Commands, channel: Res<EventReceiver>) {
	while let Ok(event) = channel.events_receiver.try_recv() {
		match event {
			Events::LobbyCreated(lobby_id) => commands.trigger(LobbyCreated(lobby_id)),
			Events::LobbyCreationFail(err) => commands.trigger(LobbyCreationFail(err)),
			Events::LobbyJoined(lobby_id) => commands.trigger(LobbyJoined(lobby_id)),
			Events::LobbyJoinFail => commands.trigger(LobbyJoinFail),
			Events::LobbyListUpdated(list) => commands.trigger(LobbyListUpdated(list)),
			Events::LobbyListUpdateFail(err) => commands.trigger(LobbyListUpdateFail(err)),
		}
	}
}

impl SteamClient {
	pub fn create_lobby(&self, lobby_type: LobbyType, max_members: u32) {
		let matchmaking = self.client.matchmaking();
		let events_sender = self.events_sender.clone();

		matchmaking.create_lobby(lobby_type, max_members, move |result: Result<steamworks::LobbyId, _>| {
			let _ = match result {
				Ok(lobby_id) => events_sender.send(Events::LobbyCreated(lobby_id)),
				Err(err) => events_sender.send(Events::LobbyCreationFail(err)),
			};
		});
	}
	pub fn join_lobby(&self, lobby_id: LobbyId) {
		let matchmaking = self.client.matchmaking();
		let events_sender = self.events_sender.clone();

		matchmaking.join_lobby(lobby_id, move |result| {
			let _ = match result {
				Ok(lobby_id) => events_sender.send(Events::LobbyJoined(lobby_id)),
				Err(_) => events_sender.send(Events::LobbyJoinFail),
			};
		});
	}
	pub fn request_lobby_list(&self) {
		let matchmaking = self.client.matchmaking();
		let events_sender = self.events_sender.clone();

		matchmaking.request_lobby_list(move |result| {
			let _ = match result {
				Ok(list) => events_sender.send(Events::LobbyListUpdated(list)),
				Err(err) => events_sender.send(Events::LobbyListUpdateFail(err)),
			};
		});
	}
	pub fn send_message_to_user(&self, networking_identity: NetworkingIdentity, send_flags: SendFlags, data: &[u8], channel: u32) -> Result<(), SteamError> {
		let networking_messages = self.client.networking_messages();

		Ok(networking_messages.send_message_to_user(networking_identity, send_flags, data, channel)?)
	}
	pub fn receive_messages_on_channel(&self, channel: u32, batch_size: usize) -> Vec<NetworkingMessage> {
		let networking_messages = self.client.networking_messages();
		networking_messages.receive_messages_on_channel(channel, batch_size)
	}
}
