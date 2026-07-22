use crate::*;

use bevy::camera;
use bevy::math;
use bevy::prelude::*;

const MAX_SHADOW_LIGHTS: usize = 24;
const LIGHT_INTENSITY: f32 = 750000.0;
const LIGHT_RANGE: f32 = 10.0;

pub struct GamePlugin;

#[derive(Debug, Clone, Copy, Component, Reflect)]
#[reflect(Component)]
struct SpawnLight;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component, wfc::Tiled, serde::Serialize, serde::Deserialize)]
#[file = "assets/tiled/tiled.json"]
pub enum Tile {
	Empty = 0,
	TopRight = 1,
	TopLeft = 2,
	DownLeft = 3,
	DownRight = 4,
	Horizontal = 5,
	Vertical = 6,
	EmptyWithLight = 7,
}

impl Plugin for GamePlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(OnEnter(GameState::InGame), setup);
		app.add_systems(Update, update_light_shadows.run_if(in_state(GameState::InGame)));
		app.add_observer(despawn_players.run_if(in_state(GameState::InGame)));
		app.add_observer(on_network_message.run_if(in_state(GameState::InGame)));
		app.add_observer(spawn_lights.run_if(in_state(GameState::InGame)));
	}
}

fn update_light_shadows(player: Single<&GlobalTransform, With<player::LocalPlayer>>, lights: Query<(&GlobalTransform, &mut SpotLight)>) {
	let position = player.translation();

	let mut lights: Vec<_> = lights.into_iter().map(|(transform, light)| (transform.translation().distance_squared(position), light)).collect();

	lights.sort_by(|a, b| a.0.total_cmp(&b.0));

	for (i, (_, mut light)) in lights.into_iter().enumerate() {
		light.shadow_maps_enabled = i < MAX_SHADOW_LIGHTS;
	}
}

fn spawn_lights(event: On<Add, SpawnLight>, transforms: Query<&Transform>, mut commands: Commands) {
	let transform = transforms.get(event.entity).copied().unwrap_or_default();

	commands.entity(event.entity).insert((
		SpotLight {
			intensity: LIGHT_INTENSITY,
			range: LIGHT_RANGE,
			inner_angle: 1.2f32,
			outer_angle: 1.50f32,
			contact_shadows_enabled: false,
			shadow_maps_enabled: false,
			..Default::default()
		},
		Transform {
			translation: transform.translation,
			scale: transform.scale,
			rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
		},
		camera::visibility::VisibilityRange {
			start_margin: 0.0..0.0,
			end_margin: 34.0..36.0,
			use_aabb: false,
		},
	));
}

fn despawn_players(event: On<steam::LobbyChatUpdate>, players: Query<(Entity, &player::Player)>, mut commands: Commands) {
	match event.0.member_state_change {
		steam::ChatMemberStateChange::Left | steamworks::ChatMemberStateChange::Disconnected | steamworks::ChatMemberStateChange::Banned | steamworks::ChatMemberStateChange::Kicked => {
			let steam_id = event.0.user_changed;

			for (entity, player) in players {
				if player.0 != steam_id {
					continue;
				}

				commands.entity(entity).despawn();
			}
		}
		_ => {}
	}
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, steam: Res<steam::SteamClient>) {
	let size = math::I64Vec2::new(128, 128);

	#[cfg(debug_assertions)]
	{
		use bevy::dev_tools;
		commands.spawn(dev_tools::infinite_grid::InfiniteGrid);
	}

	let lobby_id = steam.current_lobby().unwrap();
	let member_ids = steam.lobby_members(lobby_id);
	let owner_id = steam.lobby_owner(lobby_id);
	let my_id = steam.steam_id();

	if owner_id != my_id {
		return;
	}

	let mut rng = rand::rng();

	let map = wfc::map::Map2D::<Tile>::generate(size, &mut rng);

	for x in 0..size.x {
		for y in 0..size.y {
			let tile = *map.get_cell(math::i64vec2(x, y)).unwrap();
			let id = tile as usize;

			commands.spawn((
				DespawnOnExit(GameState::InGame),
				WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(id).from_asset("models/level_0.glb"))),
				Transform::from_xyz(x as f32 * 2.0, 0f32, y as f32 * 2.0),
				camera::visibility::VisibilityRange {
					start_margin: 0.0..0.0,
					end_margin: 70.0..75.0,
					use_aabb: false,
				},
				tile,
			));
		}
	}

	let position = find_spawn(&map).unwrap();

	for member_id in member_ids {
		let transform = Transform::from_xyz((position.x * 2) as f32, 2f32, (position.y * 2) as f32);
		if my_id == member_id {
			commands.spawn((player::Player(member_id), player::LocalPlayer, transform));
		} else {
			commands.spawn((player::Player(member_id), transform));
		}
	}

	commands.trigger(networking::BroadcastMessage {
		send_flags: steam::SendFlags::RELIABLE,
		data: networking::Message::MapGenerated(map),
	});
}

fn on_network_message(event: On<networking::MessageReceive>, asset_server: Res<AssetServer>, mut commands: Commands, steam: Res<steam::SteamClient>) {
	match &event.data {
		networking::Message::MapGenerated(map) => {
			let size = map.size();
			let lobby_id = steam.current_lobby().unwrap();
			let member_ids = steam.lobby_members(lobby_id);
			let my_id = steam.steam_id();
			let position = find_spawn(map).unwrap();

			for x in 0..size.x {
				for y in 0..size.y {
					let tile = *map.get_cell(math::i64vec2(x, y)).unwrap();
					let id = tile as usize;

					commands.spawn((
						DespawnOnExit(GameState::InGame),
						WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(id).from_asset("models/level_0.glb"))),
						Transform::from_xyz(x as f32 * 2.0, 0f32, y as f32 * 2.0),
						tile,
					));
				}
			}

			for member_id in member_ids {
				let transform = Transform::from_xyz((position.x * 2) as f32, 2f32, (position.y * 2) as f32);
				if my_id == member_id {
					commands.spawn((player::Player(member_id), player::LocalPlayer, transform));
				} else {
					commands.spawn((player::Player(member_id), transform));
				}
			}
		}
		_ => {}
	}
}

fn find_spawn(map: &wfc::map::Map2D<Tile>) -> Option<math::I64Vec2> {
	let size = map.size();
	let center = size / 2;

	if matches!(map.get_cell(center), Some(Tile::EmptyWithLight)) {
		return Some(center);
	}

	let max_radius = size.x.max(size.y);

	for radius in 1..=max_radius {
		let mut pos = center + math::I64Vec2::new(-radius, -radius);

		for _ in 0..2 * radius {
			if matches!(map.get_cell(pos), Some(Tile::EmptyWithLight)) {
				return Some(pos);
			}
			pos.x += 1;
		}

		for _ in 0..2 * radius {
			if matches!(map.get_cell(pos), Some(Tile::EmptyWithLight)) {
				return Some(pos);
			}
			pos.y += 1;
		}

		for _ in 0..2 * radius {
			if matches!(map.get_cell(pos), Some(Tile::EmptyWithLight)) {
				return Some(pos);
			}
			pos.x -= 1;
		}

		for _ in 0..2 * radius {
			if matches!(map.get_cell(pos), Some(Tile::EmptyWithLight)) {
				return Some(pos);
			}
			pos.y -= 1;
		}
	}

	None
}
