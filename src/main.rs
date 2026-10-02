use bevy::prelude::*;

mod game;
mod puzzle;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Push Kitchen".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(game::GamePlugin {
            progress_file: game::default_progress_file(),
        })
        .run();
}
