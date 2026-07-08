use crate::*;

use bevy::math;
use bevy::prelude::*;

pub struct GamePlugin;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Tile {
	Empty,
	Horizontal,
	Vertical,
}

impl Plugin for GamePlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(OnEnter(GameState::InGame), setup);
	}
}

impl wfc::Tile for Tile {
	fn all() -> &'static [Self] {
		&[Self::Empty, Self::Horizontal, Self::Vertical]
	}
	fn allowed_down(&self) -> &'static [Self] {
		match self {
			Self::Horizontal => &[Self::Empty, Self::Horizontal, Self::Vertical],
			_ => Self::all(),
		}
	}
	fn allowed_left(&self) -> &'static [Self] {
		match self {
			Self::Horizontal => &[Self::Empty, Self::Horizontal, Self::Vertical],
			_ => Self::all(),
		}
	}
	fn allowed_right(&self) -> &'static [Self] {
		match self {
			Self::Horizontal => &[Self::Empty, Self::Horizontal, Self::Vertical],
			_ => Self::all(),
		}
	}
	fn allowed_up(&self) -> &'static [Self] {
		match self {
			_ => Self::all(),
		}
	}
	fn weight(&self) -> u16 {
		match self {
			Self::Empty => 60,
			Self::Vertical => 5,
			Self::Horizontal => 5,
		}
	}
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
	let size = math::I64Vec2::new(40, 40);
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
			let id = match &tile {
				Tile::Empty => 2,
				Tile::Horizontal => 3,
				Tile::Vertical => 5,
			};
			commands.spawn((
				DespawnOnExit(GameState::InGame),
				WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(id).from_asset("level_0.glb"))),
				Transform::from_xyz(x as f32 * 2.0, 0f32, y as f32 * 2.0),
			));
		}
	}
}
