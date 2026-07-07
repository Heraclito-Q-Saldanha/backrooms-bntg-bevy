use crate::*;

use bevy::color::palettes::*;
use bevy::prelude::*;

pub struct SearchLobbyPlugin;

#[derive(Component, Debug, Clone, Copy, Default)]
struct LobbyList;

impl Plugin for SearchLobbyPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(OnEnter(GameState::SearchLobby), scene.spawn());
		app.add_systems(OnEnter(GameState::SearchLobby), request_lobby_list);
		app.add_observer(on_lobby_list_updated);
	}
}

fn request_lobby_list(steam: Res<steam::SteamClient>) {
	info!("Requesting lobby list");
	steam.request_lobby_list();
}

fn on_lobby_list_updated(triger: On<steam::LobbyListUpdated>, query: Single<Entity, With<LobbyList>>, steam: Res<steam::SteamClient>, mut commands: Commands) {
	info!("Lobby list updated");

	commands.entity(*query).despawn_children();

	for lobby_id in &triger.0 {
		let Some(name) = steam.get_lobby_data(*lobby_id, "name") else {
			continue;
		};
		let id = commands.spawn_scene(lobby_entry_component(&name)).id();
		commands.entity(*query).add_child(id);
	}
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
					justify_content: JustifyContent::SpaceBetween
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
									Text("Enter")
									TextColor(tailwind::ZINC_100)
								)]
								ui::change_bg_on_pointer::<Enter>(tailwind::EMERALD_700.into())
								ui::change_bg_on_pointer::<Leave>(tailwind::EMERALD_600.into())
								on(on_button_back_system)
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

fn lobby_entry_component(label: &str) -> impl Scene {
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
	}
}

fn on_button_refresh_system(_: On<Pointer<Click>>, steam: Res<steam::SteamClient>) {
	request_lobby_list(steam);
}

fn on_button_back_system(_: On<Pointer<Press>>, mut state: ResMut<NextState<GameState>>) {
	state.set(GameState::Menu);
}
