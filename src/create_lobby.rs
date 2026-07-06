use crate::*;

use bevy::prelude::*;

pub struct CreateLobbyPlugin;

impl Plugin for CreateLobbyPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(OnEnter(GameState::CreatingLobby), scene.spawn());
		app.add_systems(OnEnter(GameState::CreatingLobby), setup);
	}
}

fn setup(mut state: ResMut<NextState<GameState>>) {
	state.set(GameState::InGame);
}

fn scene() -> impl Scene {
	bsn! {
		DespawnOnExit::<GameState>(GameState::CreatingLobby)
		Camera2d
	}
}
