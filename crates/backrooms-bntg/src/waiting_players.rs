use crate::*;

use bevy::color::palettes::tailwind;
use bevy::prelude::*;
use bevy::ui::InteractionDisabled;

pub struct WaitingPlayersPlugin;

#[derive(Component, Clone, Copy, Default)]
struct PlayerList;

#[derive(Component, Clone, Copy, Default)]
struct StartButton;

impl Plugin for WaitingPlayersPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(OnEnter(GameState::WaitingForPlayers), (scene.spawn(), update_player_list, setup_start_button).chain());
		app.add_observer(on_network_message);
		app.add_observer(lobby_update);
	}
}

fn lobby_update(_: On<steam::LobbyChatUpdate>, query: Single<Entity, With<PlayerList>>, steam: Res<steam::SteamClient>, commands: Commands) {
	info!("Updating lobby");
	update_player_list(query, steam, commands);
}

fn setup_start_button(query: Single<Entity, With<StartButton>>, steam: Res<steam::SteamClient>, mut commands: Commands) {
	let lobby_id = steam.current_lobby().unwrap();
	let my_id = steam.steam_id();
	let owner_id = steam.lobby_owner(lobby_id);

	let mut entity = commands.entity(query.entity());

	if my_id == owner_id {
		entity.remove::<InteractionDisabled>();
	} else {
		entity.insert(InteractionDisabled);
	}
}

fn update_player_list(query: Single<Entity, With<PlayerList>>, steam: Res<steam::SteamClient>, mut commands: Commands) {
	commands.entity(*query).despawn_children();

	let lobby_id = steam.current_lobby().unwrap();
	let players = steam.lobby_members(lobby_id);

	for steam_id in players {
		let friend = steam.get_friend(steam_id);
		let name = friend.nick_name().unwrap_or(friend.name());
		let id = commands.spawn_scene(player_entry_component(&name)).id();

		commands.entity(*query).add_child(id);
	}
}

fn on_network_message(event: On<networking::MessageReceive>, mut state: ResMut<NextState<GameState>>, steam: Res<steam::SteamClient>) {
	match event.data {
		networking::Message::StartGame => {
			let Some(lobby_id) = steam.current_lobby() else {
				return;
			};
			let owner_id = steam.lobby_owner(lobby_id);
			if owner_id == event.steam_id {
				state.set(GameState::InGame);
			}
		}
		_ => {}
	}
}

fn on_button_back_system(_: On<Pointer<Press>>, mut state: ResMut<NextState<GameState>>, steam: Res<steam::SteamClient>) {
	if let Some(lobby_id) = steam.current_lobby() {
		steam.leave_lobby(lobby_id);
	}
	state.set(GameState::Menu);
}

fn on_button_start_system(_: On<Pointer<Press>>, _: Single<&StartButton, Without<InteractionDisabled>>, steam: Res<steam::SteamClient>, mut state: ResMut<NextState<GameState>>, mut commands: Commands) {
	if let Some(lobby_id) = steam.current_lobby() {
		steam.set_lobby_joinable(lobby_id, false);
	}
	commands.trigger(networking::BroadcastMessage {
		send_flags: steam::SendFlags::RELIABLE,
		data: networking::Message::StartGame,
	});
	state.set(GameState::InGame);
}

fn player_entry_component(label: &str) -> impl Scene {
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
		ui::change_bg_on_pointer_if_enable::<Enter>(tailwind::EMERALD_700.into())
		ui::change_bg_on_pointer_if_enable::<Leave>(tailwind::EMERALD_600.into())
	}
}

fn scene() -> impl Scene {
	bsn! {
		DespawnOnExit::<GameState>(GameState::WaitingForPlayers)
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
						ui::change_bg_on_pointer_if_enable::<Enter>(tailwind::EMERALD_700.into())
						ui::change_bg_on_pointer_if_enable::<Leave>(tailwind::EMERALD_600.into())
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
							Text("Start")
							TextColor(tailwind::ZINC_100)
						)]
						StartButton
						ui::change_bg_on_pointer_if_enable::<Enter>(tailwind::EMERALD_700.into())
						ui::change_bg_on_pointer_if_enable::<Leave>(tailwind::EMERALD_600.into())
						on(on_button_start_system)
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
				PlayerList
				BackgroundColor(tailwind::ZINC_900)
			)
		]
	}
}
