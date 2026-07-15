pub mod create_lobby;
pub mod game;
pub mod main_menu;
pub mod networking;
pub mod player;
pub mod search_lobby;
pub mod steam;
pub mod ui;
pub mod waiting_players;

use bevy::prelude::*;

const AMBIENT_LIGHT: f32 = 0.0025;

#[derive(States, Debug, Clone, Default, PartialEq, Eq, Hash)]
pub enum GameState {
	#[default]
	Menu,
	SearchLobby,
	CreatingLobby,
	WaitingForPlayers,
	InGame,
}

fn main() {
	let mut app = App::new();

	app.add_plugins(steam::SteamPlugin);
	app.add_plugins(DefaultPlugins.set(bevy::log::LogPlugin {
		level: bevy::log::Level::INFO,
		..Default::default()
	}));
	app.add_plugins(main_menu::MainMenuPlugin);
	app.add_plugins(search_lobby::SearchLobbyPlugin);
	app.add_plugins(create_lobby::CreateLobbyPlugin);
	app.add_plugins(waiting_players::WaitingPlayersPlugin);
	app.add_plugins(game::GamePlugin);
	app.add_plugins(player::PlayerPlugin);
	app.add_plugins(networking::NetworkingPlugin);
	app.add_plugins(bevy_skein::SkeinPlugin::default());
	app.add_plugins(avian3d::PhysicsPlugins::default());

	app.insert_resource(GlobalAmbientLight { brightness: AMBIENT_LIGHT, ..default() });

	#[cfg(debug_assertions)]
	{
		use bevy::dev_tools;
		use bevy_inspector_egui::*;
		app.add_plugins(bevy_egui::EguiPlugin::default());
		app.add_plugins(quick::WorldInspectorPlugin::new());
		app.add_plugins(avian3d::debug_render::PhysicsDebugPlugin::default());
		app.add_plugins(dev_tools::infinite_grid::InfiniteGridPlugin);
	}

	app.init_state::<GameState>();

	app.run();
}
