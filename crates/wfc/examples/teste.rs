use bevy::prelude::*;
use wfc::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, wfc::Tile)]
enum TileKind {
	#[weight(2)]
	#[constraint(all=[Grass, Sand])]
	Grass,
	#[weight(1)]
	#[constraint(all=[Water, Sand])]
	Water,
	#[weight(2)]
	#[constraint(all=[Sand, Water, Grass])]
	Sand,
}

fn main() {
	App::new().add_plugins(DefaultPlugins).add_systems(Startup, setup).run();
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>) {
	commands.spawn(Camera2d);

	let map = loop {
		let seed = rand::random();
		match Map2D::<TileKind>::generate(I64Vec2::new(64, 64), seed) {
			Ok(value) => break value,
			Err(_) => continue,
		}
	};

	let size = map.size();

	for y in 0..size.y {
		for x in 0..size.x {
			let tile = map.get_tile(I64Vec2::new(x, y)).expect("tile ausente no mapa gerado");

			let color = match tile {
				TileKind::Grass => Color::srgb(0.2, 0.7, 0.2),
				TileKind::Water => Color::srgb(0.2, 0.4, 0.9),
				TileKind::Sand => Color::srgb(0.9, 0.8, 0.45),
			};

			let world_x = (x as f32 - (size.x as f32 / 2.0) + 0.5) * 8.0;
			let world_y = ((size.y as f32 / 2.0) - y as f32 - 0.5) * 8.0;

			commands.spawn((Transform::from_xyz(world_x, world_y, 0.0), Mesh2d(meshes.add(Rectangle::new(8.0, 8.0))), MeshMaterial2d(materials.add(color))));
		}
	}
}
