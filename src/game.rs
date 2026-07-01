use crate::*;

use bevy::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), scene.spawn());
    }
}

fn scene() -> impl SceneList {
    bsn_list!(Camera3d)
}
