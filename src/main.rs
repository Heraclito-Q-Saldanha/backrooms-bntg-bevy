mod create_lobby;
mod game;
mod main_menu;
mod player;
mod search_lobby;
mod steam;

use bevy::prelude::*;

#[derive(States, Debug, Clone, Default, PartialEq, Eq, Hash)]
pub enum GameState {
	#[default]
	Menu,
	SearchLobby,
	CreatingLobby,
	InGame,
}

fn main() {
	let mut app = App::new();

	app.add_plugins(steam::SteamPlugin);
	app.add_plugins(DefaultPlugins);
	app.add_plugins(main_menu::MainMenuPlugin);
	app.add_plugins(search_lobby::SearchLobbyPlugin);
	app.add_plugins(create_lobby::CreateLobbyPlugin);
	app.add_plugins(game::GamePlugin);
	app.add_plugins(player::PlayerPlugin);
	app.add_plugins(bevy_skein::SkeinPlugin::default());
	app.add_plugins(avian3d::PhysicsPlugins::default());

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
