use crate::*;

use bevy::color::palettes::*;
use bevy::prelude::*;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(OnEnter(GameState::Menu), scene.spawn());
	}
}

fn scene() -> impl Scene {
	bsn! {
		DespawnOnExit::<GameState>(GameState::Menu)
		Camera2d
		Node {
			width: percent(100),
			height: percent(100),
			align_items: AlignItems::Center,
			justify_content: JustifyContent::Center,
			flex_direction: FlexDirection::Column,
			row_gap: px(10),
		}
		Children [
			(
				button("Play", tailwind::GREEN_800.into(), tailwind::GREEN_700.into())
				on(on_button_play_system)
			),
			(
				button("Exit", tailwind::GREEN_800.into(), tailwind::GREEN_700.into())
				on(on_button_exit_system)
			),
		]
	}
}

fn button(label: &str, normal: Color, hover: Color) -> impl Scene {
	bsn! {
		Button
		Node {
			width: px(250),
			height: px(50),
			border: px(2),
			border_radius: BorderRadius::all(px(5)),
			justify_content: JustifyContent::Center,
			align_items: AlignItems::Center,
		}
		BorderColor::from(Color::BLACK)
		BackgroundColor(normal)
		Children [(
			Text(label)
			TextColor(tailwind::CYAN_50)
			TextShadow
		)]
		on(move |trigger: On<Pointer<Enter>>, mut query: Query<&mut BackgroundColor>|{
			if let Ok(mut bg) = query.get_mut(trigger.entity) {
				bg.0 = hover;
			}
		})
		on(move |trigger: On<Pointer<Out>>, mut query: Query<&mut BackgroundColor>|{
			if let Ok(mut bg) = query.get_mut(trigger.entity) {
				bg.0 = normal;
			}
		})
	}
}

fn on_button_play_system(_: On<Pointer<Press>>, mut state: ResMut<NextState<GameState>>) {
	state.set(GameState::InGame);
}

fn on_button_exit_system(_: On<Pointer<Press>>, mut exit: MessageWriter<AppExit>) {
	exit.write(AppExit::Success);
}
