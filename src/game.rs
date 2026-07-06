use crate::*;

use bevy::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(OnEnter(GameState::InGame), spawn);
	}
}

fn spawn(mut commands: Commands, asset_server: Res<AssetServer>) {
	commands.spawn((DespawnOnExit(GameState::Menu), WorldAssetRoot(asset_server.load("game.glb#Scene0"))));
}
