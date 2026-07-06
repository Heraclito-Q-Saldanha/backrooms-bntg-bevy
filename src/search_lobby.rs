use crate::*;

use bevy::prelude::*;

pub struct SearchLobbyPlugin;

impl Plugin for SearchLobbyPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(OnEnter(GameState::SearchLobby), scene.spawn());
		app.add_systems(OnEnter(GameState::SearchLobby), setup);
	}
}

fn setup(mut state: ResMut<NextState<GameState>>) {
	state.set(GameState::InGame);
}

fn scene() -> impl Scene {
	bsn! {
		DespawnOnExit::<GameState>(GameState::SearchLobby)
		Camera2d
	}
}
