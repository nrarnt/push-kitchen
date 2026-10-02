//! The Bevy layer: shows the puzzle on screen and feeds it key presses.

mod input;
mod session;
mod view;

use bevy::prelude::*;

use crate::puzzle::{self, Board};
use session::Session;

/// The first kitchen, baked into the program when it is compiled.
const FIRST_LEVEL: &str = include_str!("../../assets/levels/01.txt");

fn first_level() -> Board {
    puzzle::parse(FIRST_LEVEL).expect("the built-in level should parse")
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(view::BACKGROUND))
            .insert_resource(Session::new(first_level()))
            .add_systems(Startup, view::spawn_camera)
            .add_systems(
                Update,
                (
                    input::handle_input,
                    // Only redraw on frames where the session was touched.
                    view::draw_board.run_if(resource_changed::<Session>),
                )
                    .chain(),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::{Dir, Pos};

    /// The game without a window: just enough of Bevy to run our systems.
    fn headless_game() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<ButtonInput<KeyCode>>()
            .add_plugins(GamePlugin);
        app.update();
        app
    }

    fn drawn_count(app: &mut App) -> usize {
        app.world_mut()
            .query_filtered::<(), With<view::Drawn>>()
            .iter(app.world())
            .count()
    }

    #[test]
    fn first_level_can_be_solved() {
        use Dir::*;
        let solution = [
            Right, Right, Right, // first crate onto the top hatch
            Down, Left, Left, Down, Right, Right, // second crate along its row
            Up, Right, Down, // and down onto the bottom hatch
        ];
        let mut board = first_level();
        for dir in solution {
            board = board.step(dir).expect("every move of the solution is legal");
        }
        assert!(board.is_solved());
    }

    #[test]
    fn the_first_level_is_drawn_when_the_game_starts() {
        let mut app = headless_game();
        let squares = (first_level().width() * first_level().height()) as usize;
        assert!(drawn_count(&mut app) > squares);
    }

    #[test]
    fn pressing_a_key_moves_the_chef() {
        let mut app = headless_game();
        let start = first_level().chef();

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowDown);
        app.update();

        let chef = app.world().resource::<Session>().board().chef();
        assert_eq!(chef, Pos::new(start.x, start.y + 1));
    }

    #[test]
    fn redrawing_replaces_the_old_picture() {
        let mut app = headless_game();
        let before = drawn_count(&mut app);

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowDown);
        app.update();

        assert_eq!(drawn_count(&mut app), before);
    }
}
