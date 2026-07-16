use crate::*;

use bevy::color::palettes::tailwind;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions};

pub struct PauseMenuPlugin;

#[derive(Debug, Default, SubStates, PartialEq, Eq, Hash, Clone, Copy)]
#[source(GameState = GameState::InGame)]
pub enum IsPaused {
	#[default]
	Running,
	Paused,
}

impl Plugin for PauseMenuPlugin {
	fn build(&self, app: &mut App) {
		app.add_sub_state::<IsPaused>();
		app.add_systems(OnEnter(IsPaused::Paused), (scene.spawn(), ungrab).chain());
		app.add_systems(OnEnter(IsPaused::Running), grab);
		app.add_systems(Update, escape_handler.run_if(in_state(GameState::InGame)));
	}
}

fn scene() -> impl Scene {
	bsn! {
		DespawnOnExit::<IsPaused>(IsPaused::Paused)
		Node {
			width: percent(100),
			height: percent(100),
			align_items: AlignItems::Center,
			justify_content: JustifyContent::Center,
		}
		Children [(
			Node {
				flex_direction: FlexDirection::Column,
				align_items: AlignItems::Center,
				padding: px(24),
				row_gap: px(8),
			}
			BackgroundColor(tailwind::RED_700)
			Children [
				(
					Text("Paused")
					TextColor(tailwind::ZINC_950)
				),
				(
					Button
					Node {
						width: px(250),
						height: px(50),
						border: px(2),
						border_radius: BorderRadius::all(px(5)),
						justify_content: JustifyContent::Center,
						align_items: AlignItems::Center,
					}
					BorderColor::from(tailwind::ZINC_100)
					BackgroundColor(tailwind::EMERALD_600)
					Children [(
						Text("Continue")
						TextColor(tailwind::ZINC_100)
					)]
					ui::change_bg_on_pointer_if_enable::<Enter>(tailwind::EMERALD_700.into())
					ui::change_bg_on_pointer_if_enable::<Leave>(tailwind::EMERALD_600.into())
					on(on_button_continue_system)
				),
				(
					Button
					Node {
						width: px(250),
						height: px(50),
						border: px(2),
						border_radius: BorderRadius::all(px(5)),
						justify_content: JustifyContent::Center,
						align_items: AlignItems::Center,
					}
					BorderColor::from(tailwind::ZINC_100)
					BackgroundColor(tailwind::EMERALD_600)
					Children [(
						Text("Exit")
						TextColor(tailwind::ZINC_100)
					)]
					ui::change_bg_on_pointer_if_enable::<Enter>(tailwind::EMERALD_700.into())
					ui::change_bg_on_pointer_if_enable::<Leave>(tailwind::EMERALD_600.into())
					on(on_button_exit_system)
				),
			]
		)]
	}
}

fn on_button_continue_system(_: On<Pointer<Press>>, mut paused: ResMut<NextState<IsPaused>>) {
	paused.set(IsPaused::Running);
}

fn on_button_exit_system(_: On<Pointer<Press>>, mut game_state: ResMut<NextState<GameState>>) {
	game_state.set(GameState::Menu);
}

fn grab(mut cursor: Single<&mut CursorOptions>) {
	cursor.visible = false;
	cursor.grab_mode = CursorGrabMode::Locked;
}

fn ungrab(mut cursor: Single<&mut CursorOptions>) {
	cursor.visible = true;
	cursor.grab_mode = CursorGrabMode::None;
}

fn escape_handler(mut set_paused: ResMut<NextState<IsPaused>>, paused: Res<State<IsPaused>>, key: Res<ButtonInput<KeyCode>>) {
	if key.just_pressed(KeyCode::Escape) {
		match paused.get() {
			IsPaused::Paused => set_paused.set(IsPaused::Running),
			IsPaused::Running => set_paused.set(IsPaused::Paused),
		}
	}
}
