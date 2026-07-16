use crate::*;

use bevy::color::palettes::tailwind;
use bevy::prelude::*;

pub struct PlayerMenuPlugin;

#[derive(Component, Default, Clone)]
struct PlayerList;

impl Plugin for PlayerMenuPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(OnEnter(ActiveMenu::PlayerMenu), (scene.spawn(), populate_player_list).chain());
		app.add_systems(Update, (tab_handler, escape_handler).run_if(in_state(GameState::InGame)));
	}
}

fn scene() -> impl Scene {
	bsn! {
		DespawnOnExit::<ActiveMenu>(ActiveMenu::PlayerMenu)
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
				min_width: px(300),
			}
			BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75))
			Children [
				(
					Text("Players")
					TextColor(tailwind::ZINC_100)
				),
				(
					PlayerList
					Node {
						flex_direction: FlexDirection::Column,
						align_items: AlignItems::FlexStart,
						width: percent(100),
						row_gap: px(4),
					}
				),
			]
		)]
	}
}

fn populate_player_list(list: Single<Entity, With<PlayerList>>, steam: Res<steam::SteamClient>, mut commands: Commands) {
	let Some(lobby_id) = steam.current_lobby() else {
		return;
	};

	for steam_id in steam.lobby_members(lobby_id) {
		let friend = steam.get_friend(steam_id);
		let name = friend.nick_name().unwrap_or(friend.name());
		let id = commands
			.spawn_scene(bsn! {
				Text(name)
				TextColor(tailwind::ZINC_100)
			})
			.id();
		commands.entity(*list).add_child(id);
	}
}

fn tab_handler(mut menu: ResMut<NextState<ActiveMenu>>, current: Res<State<ActiveMenu>>, key: Res<ButtonInput<KeyCode>>) {
	if !key.just_pressed(KeyCode::Tab) {
		return;
	}
	match current.get() {
		ActiveMenu::None => menu.set(ActiveMenu::PlayerMenu),
		ActiveMenu::PlayerMenu => menu.set(ActiveMenu::None),
		_ => {} // Don't override other menus
	}
}

fn escape_handler(mut menu: ResMut<NextState<ActiveMenu>>, current: Res<State<ActiveMenu>>, key: Res<ButtonInput<KeyCode>>) {
	if key.just_pressed(KeyCode::Escape) && *current.get() == ActiveMenu::PlayerMenu {
		menu.set(ActiveMenu::None);
	}
}
