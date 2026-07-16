pub mod create_lobby;
pub mod game;
pub mod main_menu;
pub mod networking;
pub mod pause;
pub mod player;
pub mod player_menu;
pub mod search_lobby;
pub mod steam;
pub mod ui;
pub mod waiting_players;

use bevy::light;
use bevy::prelude::*;
use bevy::window;

const AMBIENT_BRIGHTNESS: f32 = 15.0;
const SHADOW_RESOLUTION: usize = 512;

#[derive(States, Debug, Clone, Default, PartialEq, Eq, Hash)]
pub enum GameState {
	#[default]
	Menu,
	SearchLobby,
	CreatingLobby,
	WaitingForPlayers,
	InGame,
}

#[derive(SubStates, Debug, Clone, Default, PartialEq, Eq, Hash)]
#[source(GameState = GameState::InGame)]
pub enum ActiveMenu {
	#[default]
	None,
	Pause,
	PlayerMenu,
}

fn main() {
	let mut app = App::new();

	app.add_plugins(steam::SteamPlugin);
	app.add_plugins(
		DefaultPlugins
			.set(bevy::log::LogPlugin {
				level: bevy::log::Level::INFO,
				..Default::default()
			})
			.set(WindowPlugin {
				primary_window: Some(Window {
					title: "backrooms bntg".into(),
					present_mode: window::PresentMode::AutoNoVsync,
					fit_canvas_to_parent: true,
					..default()
				}),
				..default()
			}),
	);
	app.add_plugins(main_menu::MainMenuPlugin);
	app.add_plugins(search_lobby::SearchLobbyPlugin);
	app.add_plugins(create_lobby::CreateLobbyPlugin);
	app.add_plugins(waiting_players::WaitingPlayersPlugin);
	app.add_plugins(game::GamePlugin);
	app.add_plugins(pause::PauseMenuPlugin);
	app.add_plugins(player::PlayerPlugin);
	app.add_plugins(player_menu::PlayerMenuPlugin);
	app.add_plugins(networking::NetworkingPlugin);
	app.add_plugins(bevy_skein::SkeinPlugin::default());
	app.add_plugins(avian3d::PhysicsPlugins::default());

	app.insert_resource(GlobalAmbientLight { brightness: AMBIENT_BRIGHTNESS, ..default() });
	app.insert_resource(light::DirectionalLightShadowMap { size: SHADOW_RESOLUTION });

	#[cfg(debug_assertions)]
	{
		use bevy::dev_tools;

		app.add_plugins(dev_tools::infinite_grid::InfiniteGridPlugin);
		app.add_plugins(dev_tools::fps_overlay::FpsOverlayPlugin::default());
	}

	#[cfg(feature = "inspector")]
	{
		//tem uma penalidade em performance

		use bevy_inspector_egui::*;
		app.add_plugins(bevy_egui::EguiPlugin::default());
		app.add_plugins(quick::WorldInspectorPlugin::new());
	}

	#[cfg(feature = "physics_debug")]
	{
		//tem uma penalidade em performance

		app.add_plugins(avian3d::debug_render::PhysicsDebugPlugin::default());
	}

	app.init_state::<GameState>();
	app.add_sub_state::<ActiveMenu>();

	app.run();
}
