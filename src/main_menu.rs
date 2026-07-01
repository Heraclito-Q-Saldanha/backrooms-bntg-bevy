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
                button("Play", tailwind::GREEN_800.into())
                on(|_: On<Pointer<Press>>, mut state: ResMut<NextState<GameState>>| {
                    state.set(GameState::InGame);
                })
                on(|_: On<Pointer<Enter>>| println!("Enter Play"))
            ),
            (
                button("Exit", tailwind::GRAY_800.into())
                on(|_: On<Pointer<Press>>, mut exit: MessageWriter<AppExit>| {
                    exit.write(AppExit::Success);
                })
                on(|_: On<Pointer<Enter>>| println!("Enter Exit"))
            ),
        ]
    }
}

fn button(label: &str, bg_color: Color) -> impl Scene {
    bsn! {
        Button
        Node {
            width: px(150),
            height: px(65),
            border: px(5),
            border_radius: BorderRadius::all(px(10)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        BorderColor::from(Color::BLACK)
        BackgroundColor(bg_color)
        Children [(
            Text(label)
            TextColor(Color::srgb(0.9, 0.9, 0.9))
            TextShadow
        )]
    }
}
