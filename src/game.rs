use crate::*;

use bevy::math;
use bevy::prelude::*;

pub struct GamePlugin;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, wfc::Tiled)]
#[file = "assets/tiled/tiled.json"]
enum Tile {
	Empty = 0,
	TopRight = 1,
	TopLeft = 2,
	DownLeft = 3,
	DownRight = 4,
	Horizontal = 5,
	Vertical = 6,
}

impl Plugin for GamePlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(OnEnter(GameState::InGame), setup);
	}
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
	let size = math::I64Vec2::new(64, 64);
	let map = loop {
		let seed = rand::random();
		match wfc::map::Map2D::<Tile>::generate(size, seed) {
			Ok(value) => break value,
			Err(_) => continue,
		}
	};

	#[cfg(debug_assertions)]
	{
		use bevy::dev_tools;
		commands.spawn(dev_tools::infinite_grid::InfiniteGrid);
	}

	commands.spawn(player::Player);

	for x in 0..size.x {
		for y in 0..size.y {
			let tile = map.get_tile(math::i64vec2(x, y)).unwrap();
			let id = tile as usize;
			commands.spawn((
				DespawnOnExit(GameState::InGame),
				WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(id).from_asset("level_0.glb"))),
				Transform::from_xyz(x as f32 * 2.0, 0f32, y as f32 * 2.0),
			));
		}
	}
}
