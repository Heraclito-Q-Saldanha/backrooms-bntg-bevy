use crate::*;

use avian3d::prelude::*;
use bevy::camera;
use bevy::math;
use bevy::prelude::*;
use rand::RngExt;

const MAX_SHADOW_LIGHTS: usize = 24;
const LIGHT_INTENSITY: f32 = 750000.0;
const LIGHT_RANGE: f32 = 45.0;
const ALMOND_WATER_SPAWN_RATE: f32 = 10.0 / 100.0;

pub struct GamePlugin;

#[derive(Debug, Clone, Copy, Component, Reflect)]
#[reflect(Component)]
struct SpawnLight;

#[derive(Debug, Clone, Copy, Component, Reflect)]
#[reflect(Component)]
struct AlmondWater;

#[derive(Debug, Clone, Copy, Component, Reflect)]
#[reflect(Component)]
struct ItemTile(math::I64Vec2);

#[derive(Debug, Default, Clone, Copy, Component)]
struct ItemPrompt;

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

#[derive(PhysicsLayer, Clone, Copy, Debug, Default)]
pub enum GameLayer {
	#[default]
	Default,
	Interactable,
}

impl Plugin for GamePlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(OnEnter(GameState::InGame), setup);
		app.add_systems(Update, update_light_shadows.run_if(in_state(GameState::InGame)));
		app.add_systems(Update, interact_item.run_if(in_state(GameState::InGame)).run_if(in_state(ActiveMenu::None)));
		app.add_systems(Update, update_item_prompt.run_if(in_state(GameState::InGame)));
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

fn interact_item(ray_hits: Query<&RayHits, With<player::PlayerInteractionRay>>, items: Query<(Entity, &ItemTile), With<AlmondWater>>, key: Res<ButtonInput<KeyCode>>, steam: Res<steam::SteamClient>, mut commands: Commands) {
	if !key.just_pressed(KeyCode::KeyE) {
		return;
	}

	let Some((entity, tile_position)) = item_hit(&ray_hits, &items) else {
		return;
	};

	let Some(lobby_id) = steam.current_lobby() else {
		commands.entity(entity).despawn();
		return;
	};

	if steam.lobby_owner(lobby_id) == steam.steam_id() {
		commands.entity(entity).despawn();
		commands.trigger(networking::BroadcastMessage {
			send_flags: steam::SendFlags::RELIABLE,
			data: networking::Message::PickupItem(tile_position),
		});
	} else {
		commands.trigger(networking::MessageSent {
			steam_id: steam.lobby_owner(lobby_id),
			send_flags: steam::SendFlags::RELIABLE,
			data: networking::Message::PickupItem(tile_position),
		});
	}
}

fn update_item_prompt(ray_hits: Query<&RayHits, With<player::PlayerInteractionRay>>, items: Query<(Entity, &ItemTile), With<AlmondWater>>, mut prompt: Single<&mut Visibility, With<ItemPrompt>>, menu: Res<State<ActiveMenu>>) {
	if *menu.get() != ActiveMenu::None {
		**prompt = Visibility::Hidden;
		return;
	}

	let is_visible = item_hit(&ray_hits, &items).is_some();
	**prompt = if is_visible { Visibility::Inherited } else { Visibility::Hidden };
}

fn item_hit(ray_hits: &Query<&RayHits, With<player::PlayerInteractionRay>>, items: &Query<(Entity, &ItemTile), With<AlmondWater>>) -> Option<(Entity, math::I64Vec2)> {
	let Ok(hits) = ray_hits.single() else {
		return None;
	};

	let nearest = hits.iter_sorted().next()?;
	let (entity, tile) = items.get(nearest.entity).ok()?;
	Some((entity, tile.0))
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
			end_margin: 20.0..25.0,
			use_aabb: false,
		},
	));
}

fn on_network_message(
	event: On<networking::MessageReceive>,
	asset_server: Res<AssetServer>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	steam: Res<steam::SteamClient>,
	items: Query<(Entity, &ItemTile), With<AlmondWater>>,
	mut commands: Commands,
) {
	match &event.data {
		networking::Message::Map(map) => {
			let size = map.size();
			let lobby_id = steam.current_lobby().unwrap();
			let member_ids = steam.lobby_members(lobby_id);
			let my_id = steam.steam_id();
			let position = find_spawn(map).unwrap();

			for x in 0..size.x {
				for y in 0..size.y {
					let position = math::i64vec2(x, y);
					let tile = map.get_tile(position).unwrap();
					let id = tile as usize;

					commands.spawn((
						DespawnOnExit(GameState::InGame),
						WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(id).from_asset("models/level_0.glb"))),
						Transform::from_xyz(x as f32 * 2.0, 0f32, y as f32 * 2.0),
						tile,
					));

					if tile == Tile::Empty && should_spawn_almond_water() {
						commands.spawn((
							AlmondWater,
							ItemTile(position),
							DespawnOnExit(GameState::InGame),
							WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/almond_water.glb"))),
							Transform::from_xyz(x as f32 * 2.0, 1.5, y as f32 * 2.0),
							Collider::sphere(0.3),
							Sensor,
							CollisionLayers::new([GameLayer::Interactable], [GameLayer::Interactable]),
						));
					}
				}
			}

			for member_id in member_ids {
				let transform = Transform::from_xyz((position.x * 2) as f32, 2f32, (position.y * 2) as f32);
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
		}
		networking::Message::PickupItem(tile_position) => {
			let Some(lobby_id) = steam.current_lobby() else {
				return;
			};

			if steam.lobby_owner(lobby_id) == steam.steam_id() {
				if event.steam_id != steam.lobby_owner(lobby_id) {
					despawn_item_at_tile(*tile_position, &items, &mut commands);
					commands.trigger(networking::BroadcastMessage {
						send_flags: steam::SendFlags::RELIABLE,
						data: networking::Message::PickupItem(*tile_position),
					});
				}
			} else {
				despawn_item_at_tile(*tile_position, &items, &mut commands);
			}
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

	commands.spawn_scene(bsn! {
		DespawnOnExit::<GameState>(GameState::InGame)
		ItemPrompt
		Node {
			width: percent(100),
			height: percent(100),
			position_type: PositionType::Absolute,
			justify_content: JustifyContent::Center,
			align_items: AlignItems::End,
			padding: UiRect::bottom(px(40)),
		}
		Visibility::Hidden
		Children [(
			Node {
				padding: px(10),
				border_radius: BorderRadius::all(px(6)),
			}
			BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8))
			Children [(
				Text("Press E to pick up almond water")
				TextColor(Color::WHITE)
			)]
		)]
	});

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

	let map = loop {
		let seed = rand::random();
		match wfc::map::Map2D::<Tile>::generate(size, seed) {
			Ok(value) => break value,
			Err(_) => continue,
		}
	};

	let size = map.size();

	for x in 0..size.x {
		for y in 0..size.y {
			let position = math::i64vec2(x, y);
			let tile = map.get_tile(position).unwrap();
			let id = tile as usize;

			commands.spawn((
				DespawnOnExit(GameState::InGame),
				WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(id).from_asset("models/level_0.glb"))),
				Transform::from_xyz(x as f32 * 2.0, 0f32, y as f32 * 2.0),
				tile,
			));

			if tile == Tile::Empty && should_spawn_almond_water() {
				commands.spawn((
					AlmondWater,
					ItemTile(position),
					DespawnOnExit(GameState::InGame),
					WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/almond_water.glb"))),
					Transform::from_xyz(x as f32 * 2.0, 0.35, y as f32 * 2.0),
					Collider::sphere(0.3),
					Sensor,
					CollisionLayers::new([GameLayer::Interactable], [GameLayer::Interactable]),
				));
			}
		}
	}

	let position = find_spawn(&map).unwrap();

	for member_id in member_ids {
		let transform = Transform::from_xyz((position.x * 2) as f32, 2f32, (position.y * 2) as f32);
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

	commands.trigger(networking::BroadcastMessage {
		send_flags: steam::SendFlags::RELIABLE,
		data: networking::Message::Map(map),
	});
}

fn find_spawn(map: &wfc::map::Map2D<Tile>) -> Option<math::I64Vec2> {
	let size = map.size();
	let center = size / 2;

	if matches!(map.get_tile(center), Some(Tile::EmptyWithLight)) {
		return Some(center);
	}

	let max_radius = size.x.max(size.y);

	for radius in 1..=max_radius {
		let mut pos = center + math::I64Vec2::new(-radius, -radius);

		for _ in 0..2 * radius {
			if matches!(map.get_tile(pos), Some(Tile::EmptyWithLight)) {
				return Some(pos);
			}
			pos.x += 1;
		}

		for _ in 0..2 * radius {
			if matches!(map.get_tile(pos), Some(Tile::EmptyWithLight)) {
				return Some(pos);
			}
			pos.y += 1;
		}

		for _ in 0..2 * radius {
			if matches!(map.get_tile(pos), Some(Tile::EmptyWithLight)) {
				return Some(pos);
			}
			pos.x -= 1;
		}

		for _ in 0..2 * radius {
			if matches!(map.get_tile(pos), Some(Tile::EmptyWithLight)) {
				return Some(pos);
			}
			pos.y -= 1;
		}
	}

	None
}

fn despawn_item_at_tile(tile_position: math::I64Vec2, items: &Query<(Entity, &ItemTile), With<AlmondWater>>, commands: &mut Commands) {
	for (entity, item_tile) in items {
		if item_tile.0 == tile_position {
			commands.entity(entity).despawn();
			break;
		}
	}
}

fn should_spawn_almond_water() -> bool {
	rand::rng().random_bool(ALMOND_WATER_SPAWN_RATE as f64)
}
