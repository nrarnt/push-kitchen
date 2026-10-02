use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;

mod game;
mod puzzle;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Push Kitchen".into(),
                        // In a browser: draw into the page's <canvas id="game">,
                        // at whatever size the page gives it.
                        canvas: Some("#game".into()),
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    // Our assets have no ".meta" files beside them. Without
                    // this, the web version would ask the server for one per
                    // asset and get an error back each time.
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }),
        )
        .add_plugins(game::GamePlugin {
            save_slot: game::SaveSlot::for_this_platform(),
        })
        .run();
}
