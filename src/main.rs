mod game;
mod main_menu;

use bevy::prelude::*;

#[derive(States, Debug, Clone, Default, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    Menu,
    InGame,
}

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins);
    app.add_plugins(main_menu::MainMenuPlugin);
    app.add_plugins(game::GamePlugin);

    #[cfg(debug_assertions)]
    {
        use bevy_inspector_egui::*;
        app.add_plugins(bevy_egui::EguiPlugin::default());
        app.add_plugins(quick::WorldInspectorPlugin::new());
    }

    app.init_state::<GameState>();

    app.run();
}
