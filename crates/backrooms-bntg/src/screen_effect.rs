use bevy::core_pipeline;
use bevy::prelude::*;
use bevy::render;
use bevy::shader;

#[derive(Clone, Copy, Component, render::extract_component::ExtractComponent, render::render_resource::ShaderType, Default)]
pub struct FullscreenEffect {
	pub intensity: f32,
}

impl core_pipeline::fullscreen_material::FullscreenMaterial for FullscreenEffect {
	fn fragment_shader() -> shader::ShaderRef {
		"shaders/fullscreen_effect.wgsl".into()
	}
}
