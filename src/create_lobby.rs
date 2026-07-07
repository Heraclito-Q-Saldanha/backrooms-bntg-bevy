use crate::*;

use bevy::color::palettes::*;
use bevy::prelude::*;
use bevy::text;

pub struct CreateLobbyPlugin;

impl Plugin for CreateLobbyPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(OnEnter(GameState::CreatingLobby), scene.spawn());
		app.add_observer(on_lobby_created);
	}
}

fn scene() -> impl Scene {
	bsn! {
		DespawnOnExit::<GameState>(GameState::CreatingLobby)
		Camera2d
		Node {
			width: percent(100),
			height: percent(100),
			row_gap: px(10),
			align_items: AlignItems::Center,
			justify_content: JustifyContent::Center,
			flex_direction: FlexDirection::Column,
		}
		BackgroundColor(tailwind::ZINC_950)
		Children [
			(
				Node {
					width: px(250),
					height: px(40),
					border: px(2),
					border_radius: BorderRadius::all(px(5)),
					justify_content: JustifyContent::Center,
					align_items: AlignItems::Center,
				}
				BorderColor::from(tailwind::ZINC_100)
				text::EditableText {
					allow_newlines: false,
				}
				text::TextCursorStyle
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
					Text("Create")
					TextColor(tailwind::ZINC_100)
				)]
				ui::change_bg_on_pointer::<Enter>(tailwind::EMERALD_700.into())
				ui::change_bg_on_pointer::<Leave>(tailwind::EMERALD_600.into())
				on(on_create_button_click)
			)
		]
	}
}

fn on_create_button_click(_: On<Pointer<Click>>, steam: Res<steam::SteamClient>) {
	info!("Creating lobby");
	steam.create_lobby(steamworks::LobbyType::Public, 10);
}

fn on_lobby_created(event: On<steam::LobbyCreated>, mut state: ResMut<NextState<GameState>>) {
	info!(r#"Lobby "{}" created"#, event.0.raw());
	state.set(GameState::InGame);
}
