use crate::*;

use bevy::math;
use bevy::prelude::*;

const MAX_SHADOW_LIGHTS: usize = 64;
const LIGHT_INTENSITY: f32 = 750000.0;
const LIGHT_RANGE: f32 = 35.0;

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
	));
}

fn on_network_message(event: On<networking::MessageReceive>, asset_server: Res<AssetServer>, mut commands: Commands) {
	match &event.data {
		networking::Message::Map(map) => {
			spawn_map(map, asset_server, &mut commands);
		}
		_ => {}
	}
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

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>, steam: Res<steam::SteamClient>) {
	let size = math::I64Vec2::new(64, 64);

	#[cfg(debug_assertions)]
	{
		use bevy::dev_tools;
		commands.spawn(dev_tools::infinite_grid::InfiniteGrid);
	}

	let lobby_id = steam.current_lobby().unwrap();
	let member_ids = steam.lobby_members(lobby_id);
	let owner_id = steam.lobby_owner(lobby_id);
	let my_id = steam.steam_id();

	for member_id in member_ids {
		let transform = Transform::from_xyz(size.x as f32, 1.60f32, size.y as f32);
		if my_id == member_id {
			commands.spawn((
				Mesh3d(meshes.add(Mesh::from(Capsule3d::default()))),
				MeshMaterial3d(materials.add(Color::from(Srgba::WHITE))),
				player::Player(member_id),
				player::LocalPlayer,
				transform,
			));
		} else {
			commands.spawn((Mesh3d(meshes.add(Mesh::from(Capsule3d::default()))), MeshMaterial3d(materials.add(Color::from(Srgba::BLUE))), player::Player(member_id), transform));
		}
	}

	if owner_id != my_id {
		return;
	}

	let map = loop {
		let seed = rand::random();
		match wfc::map::Map2D::<Tile>::generate(size, seed) {
			Ok(value) => break value,
			Err(_) => continue,
		}
	};

	spawn_map(&map, asset_server, &mut commands);

	commands.trigger(networking::BroadcastMessage {
		send_flags: steam::SendFlags::RELIABLE,
		data: networking::Message::Map(map),
	});
}

fn spawn_map(map: &wfc::map::Map2D<Tile>, asset_server: Res<AssetServer>, commands: &mut Commands) {
	let size = map.size();
	for x in 0..size.x {
		for y in 0..size.y {
			let tile = map.get_tile(math::i64vec2(x, y)).unwrap();
			let id = tile as usize;
			commands.spawn((
				DespawnOnExit(GameState::InGame),
				WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(id).from_asset("level_0.glb"))),
				Transform::from_xyz(x as f32 * 2.0, 0f32, y as f32 * 2.0),
				tile,
			));
		}
	}
}
