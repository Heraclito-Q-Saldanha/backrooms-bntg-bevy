use crate::*;

use avian3d::prelude::*;
use bevy::anti_alias;
use bevy::camera;
use bevy::core_pipeline;
use bevy::input;
use bevy::light;
use bevy::pbr;
use bevy::post_process;
use bevy::prelude::*;

const PITCH_LIMIT: f32 = std::f32::consts::FRAC_PI_2 - 0.01;
const BLOOM_INTENSITY: f32 = 0.35;

const CAMERA_SENSITIVITY: Vec2 = Vec2 { x: 0.003, y: 0.002 };

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Update, movement_player.run_if(in_state(GameState::InGame)).run_if(in_state(ActiveMenu::None)));
		app.add_systems(Update, camera_player.run_if(in_state(GameState::InGame)).run_if(in_state(ActiveMenu::None)));
		app.add_systems(Update, sync_position.run_if(in_state(GameState::InGame)));
		app.add_observer(on_network_message);
		app.add_observer(config_local_player);
		app.add_observer(config_player);
	}
}

#[derive(Debug, Component)]
#[require(Transform)]
pub struct Player(pub steam::SteamId);

#[derive(Debug, Component)]
#[require(Transform, PlayerSpeed)]
pub struct LocalPlayer;

#[derive(Debug, Component)]
pub struct PlayerSpeed(f32);

fn config_local_player(event: On<Add, LocalPlayer>, mut commands: Commands) {
	commands.entity(event.entity).insert((
		RigidBody::Dynamic,
		Collider::capsule(0.2, 1.0),
		LinearVelocity::ZERO,
		LockedAxes::ROTATION_LOCKED,
		TransformInterpolation,
		children![(
			Transform::from_translation(Vec3::Y * 0.3),
			Camera3d::default(),
			camera::Hdr,
			core_pipeline::tonemapping::Tonemapping::TonyMcMapface,
			post_process::bloom::Bloom {
				intensity: BLOOM_INTENSITY,
				..Default::default()
			},
			pbr::ScreenSpaceAmbientOcclusion::default(),
			anti_alias::taa::TemporalAntiAliasing::default(),
			Msaa::Off,
			post_process::effect_stack::ChromaticAberration { intensity: 0.015, ..Default::default() },
			post_process::effect_stack::LensDistortion { intensity: 0.10, ..Default::default() },
			post_process::effect_stack::Vignette { intensity: 0.9, ..Default::default() },
		)],
		#[cfg(feature = "inspector")]
		{
			bevy_inspector_egui::bevy_egui::PrimaryEguiContext
		},
	));
}

fn config_player(event: On<Add, Player>, mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
	commands
		.entity(event.entity)
		.insert((light::NotShadowCaster, Mesh3d(meshes.add(Mesh::from(Capsule3d::default()))), MeshMaterial3d(materials.add(Color::from(Srgba::BLUE)))));
}

fn movement_player(query: Single<(&PlayerSpeed, &mut LinearVelocity), With<LocalPlayer>>, camera: Single<&Transform, (With<Camera3d>, Without<LocalPlayer>)>, keys: Res<ButtonInput<KeyCode>>) {
	let (speed, mut velocity) = query.into_inner();
	let camera = camera.into_inner();

	let x = (keys.pressed(KeyCode::KeyD) as i8 - keys.pressed(KeyCode::KeyA) as i8) as f32;
	let z = (keys.pressed(KeyCode::KeyS) as i8 - keys.pressed(KeyCode::KeyW) as i8) as f32;

	let input = Vec3::new(x, 0.0, z);

	if input.length_squared() > 0.0 {
		let input = input.normalize();

		let (yaw, _, _) = camera.rotation.to_euler(EulerRot::YXZ);
		let direction = Quat::from_rotation_y(yaw) * input;

		velocity.x = (direction.x * speed.0) as f64;
		velocity.z = (direction.z * speed.0) as f64;
	} else {
		velocity.x = 0.0;
		velocity.z = 0.0;
	}
}

fn sync_position(player: Single<&Transform, With<LocalPlayer>>, mut commands: Commands) {
	let player = player.into_inner();
	commands.trigger(networking::BroadcastMessage {
		send_flags: steam::SendFlags::UNRELIABLE_NO_NAGLE,
		data: networking::Message::PlayerPosition(player.translation),
	});
}

fn camera_player(accumulated_mouse_motion: Res<input::mouse::AccumulatedMouseMotion>, camera: Single<&mut Transform, With<Camera3d>>) {
	let delta = accumulated_mouse_motion.delta;

	if delta == Vec2::ZERO {
		return;
	}

	let delta_yaw = -delta.x * CAMERA_SENSITIVITY.x;
	let delta_pitch = -delta.y * CAMERA_SENSITIVITY.y;

	let mut camera = camera.into_inner();

	let (mut yaw, mut pitch, _) = camera.rotation.to_euler(EulerRot::YXZ);

	pitch = (pitch + delta_pitch).clamp(-PITCH_LIMIT, PITCH_LIMIT);
	yaw += delta_yaw;

	camera.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
}

fn on_network_message(event: On<networking::MessageReceive>, players: Query<(&mut Transform, &mut Player)>) {
	match event.data {
		networking::Message::PlayerPosition(position) => {
			let steam_id = event.steam_id;

			for (mut transform, player) in players {
				if player.0 != steam_id {
					continue;
				}
				transform.translation = position;
			}
		}
		_ => {}
	}
}

impl Default for PlayerSpeed {
	#[inline(always)]
	fn default() -> Self {
		Self(4.0)
	}
}
