use crate::*;

use bevy::color::palettes::*;
use bevy::prelude::*;
use bevy::text;

pub struct SearchLobbyPlugin;

#[derive(Component, Debug, Clone, Copy, Default)]
struct LobbyList;

#[derive(Component, Debug, Clone, Copy, Default)]
struct InputLobby;

impl Plugin for SearchLobbyPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(OnEnter(GameState::SearchLobby), scene.spawn());
		app.add_systems(OnEnter(GameState::SearchLobby), request_lobby_list);
		app.add_observer(on_lobby_list_updated);
		app.add_observer(on_lobby_joined);
	}
}

fn on_lobby_list_updated(triger: On<steam::LobbyListUpdated>, query: Single<Entity, With<LobbyList>>, steam: Res<steam::SteamClient>, mut commands: Commands) {
	info!("Lobby list updated");
	commands.entity(*query).despawn_children();

	for lobby_id in &triger.0 {
		let Some(name) = steam.get_lobby_data(*lobby_id, "name") else {
			continue;
		};
		let id = commands.spawn_scene(lobby_entry_component(&name, *lobby_id)).id();
		commands.entity(*query).add_child(id);
	}
}

fn on_lobby_joined(event: On<steam::LobbyJoined>, mut state: ResMut<NextState<GameState>>) {
	let lobby_id = event.0;
	info!(r#"Lobby "{}" joined"#, lobby_id.raw());

	state.set(GameState::InGame);
}

fn request_lobby_list(steam: Res<steam::SteamClient>) {
	info!("Requesting lobby list");
	steam.request_lobby_list();
}

fn scene() -> impl Scene {
	bsn! {
		DespawnOnExit::<GameState>(GameState::SearchLobby)
		Camera2d
		Node {
			width: percent(100),
			height: percent(100),
			padding: px(24),
			flex_direction: FlexDirection::Column,
			justify_content: JustifyContent::Center,
			align_items: AlignItems::Center,
			row_gap: px(16),
			overflow: Overflow::scroll(),
		}
		BackgroundColor(tailwind::ZINC_950)
		Children [
			(
				Node {
					width: percent(100),
					height: px(100),
					flex_direction: FlexDirection::Row,
					justify_content: JustifyContent::SpaceBetween,
					align_items: AlignItems::Center
				}
				Children [
					(
						Button
						Node {
							width: px(100),
							height: px(50),
							border: px(2),
							border_radius: BorderRadius::all(px(5)),
							justify_content: JustifyContent::Center,
							align_items: AlignItems::Center,
						}
						BorderColor::from(tailwind::ZINC_100)
						BackgroundColor(tailwind::EMERALD_600)
						Children [(
							Text("Back")
							TextColor(tailwind::ZINC_100)
						)]
						ui::change_bg_on_pointer::<Enter>(tailwind::EMERALD_700.into())
						ui::change_bg_on_pointer::<Leave>(tailwind::EMERALD_600.into())
						on(on_button_back_system)
					),
					(
						Node {
							column_gap: px(16),
							flex_direction: FlexDirection::Row,
							align_items: AlignItems::Center
						}
						Children [
							(
								InputLobby
								text::EditableText {
									allow_newlines: false,
								}
								text::TextCursorStyle
								Node {
									width: px(150),
									height: px(50),
									border: px(2),
									border_radius: BorderRadius::all(px(5)),
									justify_content: JustifyContent::Center,
									align_items: AlignItems::Center,
								}
								BorderColor::from(tailwind::ZINC_100)
							),
							(
								Button
								Node {
									width: px(100),
									height: px(50),
									border: px(2),
									border_radius: BorderRadius::all(px(5)),
									justify_content: JustifyContent::Center,
									align_items: AlignItems::Center,
								}
								BorderColor::from(tailwind::ZINC_100)
								BackgroundColor(tailwind::EMERALD_600)
								Children [(
									Text("Enter")
									TextColor(tailwind::ZINC_100)
								)]
								ui::change_bg_on_pointer::<Enter>(tailwind::EMERALD_700.into())
								ui::change_bg_on_pointer::<Leave>(tailwind::EMERALD_600.into())
								on(on_button_enter_system)
							),
							(
								Button
								Node {
									width: px(100),
									height: px(50),
									border: px(2),
									border_radius: BorderRadius::all(px(5)),
									justify_content: JustifyContent::Center,
									align_items: AlignItems::Center,
								}
								BorderColor::from(tailwind::ZINC_100)
								BackgroundColor(tailwind::EMERALD_600)
								Children [(
									Text("Refresh")
									TextColor(tailwind::ZINC_100)
								)]
								ui::change_bg_on_pointer::<Enter>(tailwind::EMERALD_700.into())
								ui::change_bg_on_pointer::<Leave>(tailwind::EMERALD_600.into())
								on(on_button_refresh_system)
							)
						]
					)
				]
			),
			(
				Node {
					width: percent(100),
					height: percent(100),
					row_gap: px(8),
					column_gap: px(8),
					padding: px(16),
					border: px(2),
					border_radius: BorderRadius::all(px(12)),
					flex_direction: FlexDirection::Column,
					align_items: AlignItems::Center,
					overflow: Overflow::scroll(),
				}
				LobbyList
				BackgroundColor(tailwind::ZINC_900)
			)
		]
	}
}

fn lobby_entry_component(label: &str, lobby_id: steam::LobbyId) -> impl Scene {
	bsn! {
		Button
		Node {
			width: percent(100),
			height: px(50),
			border: px(2),
			border_radius: BorderRadius::all(px(5)),
			justify_content: JustifyContent::Center,
			align_items: AlignItems::Center,
		}
		BorderColor::from(tailwind::ZINC_100)
		BackgroundColor(tailwind::EMERALD_600)
		Children [(
			Text(label)
			TextColor(tailwind::ZINC_100)
		)]
		ui::change_bg_on_pointer::<Enter>(tailwind::EMERALD_700.into())
		ui::change_bg_on_pointer::<Leave>(tailwind::EMERALD_600.into())
		on(move |_: On<Pointer<Click>>, mut query: Single<&mut text::EditableText, With<InputLobby>>| {
			let text = lobby_id.raw().to_string();
			query.editor_mut().set_text(&text);
		})
	}
}

fn on_button_refresh_system(_: On<Pointer<Click>>, steam: Res<steam::SteamClient>) {
	request_lobby_list(steam);
}

fn on_button_enter_system(_: On<Pointer<Click>>, steam: Res<steam::SteamClient>, input: Single<&mut text::EditableText, With<InputLobby>>) {
	let Ok(value) = input.value().to_string().parse::<u64>() else {
		info!("You try parse a non interger value");
		return;
	};
	let lobby_id = steam::LobbyId::from_raw(value);
	steam.join_lobby(lobby_id);
}

fn on_button_back_system(_: On<Pointer<Press>>, mut state: ResMut<NextState<GameState>>) {
	state.set(GameState::Menu);
}
