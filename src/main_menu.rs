use crate::*;

use bevy::color::palettes::*;
use bevy::prelude::*;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(OnEnter(GameState::Menu), scene.spawn());
	}
}

fn scene() -> impl Scene {
	bsn! {
		DespawnOnExit::<GameState>(GameState::Menu)
		Camera2d
		Node {
			width: percent(100),
			height: percent(100),
			row_gap: px(10),
			align_items: AlignItems::Center,
			justify_content: JustifyContent::Center,
			flex_direction: FlexDirection::Column,
		}
		BackgroundColor(tailwind::ZINC_950)
		Children [
			(
				ui::button("Create Lobby", tailwind::EMERALD_600.into(), tailwind::EMERALD_700.into())
				on(on_button_create_lobby_system)
			),
			(
				ui::button("Join Lobby", tailwind::EMERALD_600.into(), tailwind::EMERALD_700.into())
				on(on_button_join_lobby_system)
			),
			(
				ui::button("Exit", tailwind::EMERALD_600.into(), tailwind::EMERALD_700.into())
				on(on_button_exit_system)
			),
		]
	}
}

fn on_button_create_lobby_system(_: On<Pointer<Press>>, mut state: ResMut<NextState<GameState>>) {
	state.set(GameState::CreatingLobby);
}

fn on_button_join_lobby_system(_: On<Pointer<Press>>, mut state: ResMut<NextState<GameState>>) {
	state.set(GameState::SearchLobby);
}

fn on_button_exit_system(_: On<Pointer<Press>>, mut exit: MessageWriter<AppExit>) {
	exit.write(AppExit::Success);
}
