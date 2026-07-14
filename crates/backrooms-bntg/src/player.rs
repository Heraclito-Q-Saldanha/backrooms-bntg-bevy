use crate::*;

use bevy::input;
use bevy::prelude::*;

const PITCH_LIMIT: f32 = std::f32::consts::FRAC_PI_2 - 0.01;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Update, movement_player.run_if(in_state(GameState::InGame)));
		app.add_systems(Update, camera_player.run_if(in_state(GameState::InGame)));
		app.add_observer(on_network_message);
	}
}

#[derive(Debug, Component)]
#[require(Transform)]
pub struct Player(pub steam::SteamId);

#[derive(Debug, Component)]
#[require(Transform, Camera3d, PlayerSpeed, CameraSensitivity)]
pub struct LocalPlayer;

#[derive(Debug, Component, Reflect, Deref, DerefMut)]
#[reflect(Component)]
struct CameraSensitivity(Vec2);

#[derive(Debug, Component)]
pub struct PlayerSpeed(f32);

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

fn movement_player(query: Single<(&mut PlayerSpeed, &mut Transform), With<LocalPlayer>>, keys: Res<ButtonInput<KeyCode>>, time: Res<Time>, mut commands: Commands) {
	let (speed, mut transform) = query.into_inner();

	let delta = time.delta_secs();

	let x = ((keys.pressed(KeyCode::KeyD) as i8) - (keys.pressed(KeyCode::KeyA) as i8)) as f32;
	let z = ((keys.pressed(KeyCode::KeyS) as i8) - (keys.pressed(KeyCode::KeyW) as i8)) as f32;

	let input = Vec3::new(x, 0.0, z);

	if input.length_squared() > 0.0 {
		let input = input.normalize();
		let direction = transform.rotation * input;
		transform.translation += direction * delta * speed.0;

		commands.trigger(networking::BroadcastMessage {
			send_flags: steam::SendFlags::NO_NAGLE | steam::SendFlags::UNRELIABLE,
			data: networking::Message::Position(transform.translation),
		});
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
