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

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Update, movement_player.run_if(in_state(GameState::InGame)).run_if(in_state(ActiveMenu::None)));
		app.add_systems(Update, camera_player.run_if(in_state(GameState::InGame)).run_if(in_state(ActiveMenu::None)));
		app.add_observer(on_network_message);
		app.add_observer(config_local_player);
		app.add_observer(config_player);
	}
}

#[derive(Debug, Component)]
#[require(Transform)]
pub struct Player(pub steam::SteamId);

#[derive(Debug, Component)]
#[require(Transform, PlayerSpeed, CameraSensitivity)]
pub struct LocalPlayer;

#[derive(Debug, Component, Reflect, Deref, DerefMut)]
#[reflect(Component)]
struct CameraSensitivity(Vec2);

#[derive(Debug, Component)]
pub struct PlayerSpeed(f32);

fn config_local_player(event: On<Add, LocalPlayer>, mut commands: Commands) {
	commands.entity(event.entity).insert((
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
		RigidBody::Dynamic,
		Collider::capsule(0.1, 0.5),
		LinearVelocity::ZERO,
		LockedAxes::ROTATION_LOCKED,
		TransformInterpolation,
		#[cfg(feature = "inspector")]
		{
			bevy_inspector_egui::bevy_egui::PrimaryEguiContext
		},
	));
}

fn config_player(event: On<Add, Player>, mut commands: Commands) {
	commands.entity(event.entity).insert((light::NotShadowCaster,));
}

fn on_network_message(event: On<networking::MessageReceive>, players: Query<(&mut Transform, &mut Player)>) {
	match event.data {
		networking::Message::Position(position) => {
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

fn movement_player(query: Single<(&PlayerSpeed, &Transform, &mut LinearVelocity), With<LocalPlayer>>, keys: Res<ButtonInput<KeyCode>>) {
	let (speed, transform, mut velocity) = query.into_inner();

	let x = (keys.pressed(KeyCode::KeyD) as i8 - keys.pressed(KeyCode::KeyA) as i8) as f32;
	let z = (keys.pressed(KeyCode::KeyS) as i8 - keys.pressed(KeyCode::KeyW) as i8) as f32;

	let input = Vec3::new(x, 0.0, z);

	if input.length_squared() > 0.0 {
		let input = input.normalize();
		let direction = transform.rotation * input;

		velocity.x = (direction.x * speed.0) as f64;
		velocity.z = (direction.z * speed.0) as f64;
	} else {
		velocity.x = 0.0;
		velocity.z = 0.0;
	}
}

fn camera_player(accumulated_mouse_motion: Res<input::mouse::AccumulatedMouseMotion>, player: Single<(&mut Transform, &CameraSensitivity), With<LocalPlayer>>) {
	let (mut transform, camera_sensitivity) = player.into_inner();

	let delta = accumulated_mouse_motion.delta;

	if delta != Vec2::ZERO {
		let delta_yaw = -delta.x * camera_sensitivity.x;
		let delta_pitch = -delta.y * camera_sensitivity.y;

		let (yaw, pitch, roll) = transform.rotation.to_euler(EulerRot::YXZ);
		let yaw = yaw + delta_yaw;

		let pitch = (pitch + delta_pitch).clamp(-PITCH_LIMIT, PITCH_LIMIT);

		transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
	}
}

impl Default for CameraSensitivity {
	#[inline]
	fn default() -> Self {
		Self(Vec2::new(0.003, 0.002))
	}
}

impl Default for PlayerSpeed {
	#[inline(always)]
	fn default() -> Self {
		Self(4.0)
	}
}
