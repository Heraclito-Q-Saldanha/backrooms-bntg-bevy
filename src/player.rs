use crate::*;

use bevy::input;
use bevy::prelude::*;

const PITCH_LIMIT: f32 = std::f32::consts::FRAC_PI_2 - 0.01;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Update, rotate_camera.run_if(in_state(GameState::InGame)));
	}
}

#[derive(Debug, Component, Reflect, Deref, DerefMut)]
#[reflect(Component)]
struct CameraSensitivity(Vec2);

fn rotate_camera(accumulated_mouse_motion: Res<input::mouse::AccumulatedMouseMotion>, player: Single<(&mut Transform, &CameraSensitivity), With<Camera>>) {
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
