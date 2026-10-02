//! The Bevy layer: shows the puzzle on screen and feeds it key presses.

mod input;
mod levels;
mod progress;
mod session;
mod ui;
mod view;

use std::path::PathBuf;

use bevy::prelude::*;

use levels::LEVELS;
use progress::{Progress, ProgressFile};
use session::Session;

pub use progress::default_progress_file;

/// Which screen the game is showing.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Screen {
    #[default]
    Menu,
    Playing,
}

/// The level picked in the menu: its place in `LEVELS`. While playing, this
/// is the level being played.
#[derive(Resource)]
pub struct Selected(pub usize);

/// The level the menu should start on: the first one still to be solved.
fn first_unsolved(progress: &Progress) -> usize {
    LEVELS
        .iter()
        .position(|level| !progress.is_solved(level.id))
        .unwrap_or(0)
}

/// Puts the selected level on the table, fresh.
fn start_level(mut commands: Commands, selected: Res<Selected>) {
    commands.insert_resource(Session::new(LEVELS[selected.0].board()));
}

/// Writes the level down as solved, the first time it is.
fn record_solved(
    session: Res<Session>,
    selected: Res<Selected>,
    mut progress: ResMut<Progress>,
    file: Res<ProgressFile>,
) {
    if session.board().is_solved()
        && progress.mark_solved(LEVELS[selected.0].id)
        && let Err(error) = progress.save(&file.0)
    {
        // Not being able to save should not stop the game.
        warn!("could not save progress to {}: {error}", file.0.display());
    }
}

pub struct GamePlugin {
    pub progress_file: PathBuf,
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        let progress = Progress::load(&self.progress_file).unwrap_or_else(|error| {
            warn!("could not read {}: {error}", self.progress_file.display());
            Progress::default()
        });

        app.init_state::<Screen>()
            .insert_resource(ClearColor(view::BACKGROUND))
            .insert_resource(Selected(first_unsolved(&progress)))
            .insert_resource(progress)
            .insert_resource(ProgressFile(self.progress_file.clone()))
            .add_systems(Startup, view::spawn_camera)
            .add_systems(OnEnter(Screen::Menu), ui::draw_menu)
            .add_systems(OnEnter(Screen::Playing), start_level)
            .add_systems(
                Update,
                (
                    (
                        ui::handle_menu_input,
                        ui::draw_menu.run_if(resource_changed::<Selected>),
                    )
                        .chain()
                        .run_if(in_state(Screen::Menu)),
                    (
                        input::handle_input,
                        // Only on frames where the session was touched.
                        (record_solved, view::draw_board)
                            .chain()
                            .run_if(resource_exists_and_changed::<Session>),
                    )
                        .chain()
                        .run_if(in_state(Screen::Playing)),
                ),
            );
    }
}

#[cfg(test)]
mod tests {
    use bevy::state::app::StatesPlugin;

    use super::*;
    use crate::puzzle::Pos;
    use progress::scratch_file;

    /// The keys that solve the first level.
    const FIRST_LEVEL_SOLUTION: [KeyCode; 5] = [
        KeyCode::ArrowDown,
        KeyCode::ArrowRight,
        KeyCode::ArrowRight,
        KeyCode::ArrowUp,
        KeyCode::ArrowUp,
    ];

    /// The game without a window: just enough of Bevy to run our systems.
    fn headless_game(progress_file: PathBuf) -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_resource::<ButtonInput<KeyCode>>()
            .add_plugins(GamePlugin { progress_file });
        app.update();
        app
    }

    /// Presses a key and lets it go again, with one frame for each.
    fn tap(app: &mut App, key: KeyCode) {
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(key);
        app.update();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        app.update();
    }

    fn screen(app: &App) -> Screen {
        *app.world().resource::<State<Screen>>().get()
    }

    fn selected(app: &App) -> usize {
        app.world().resource::<Selected>().0
    }

    fn drawn_count(app: &mut App) -> usize {
        app.world_mut()
            .query_filtered::<(), With<view::Drawn>>()
            .iter(app.world())
            .count()
    }

    #[test]
    fn the_game_starts_in_the_menu() {
        let mut app = headless_game(scratch_file("starts-in-menu"));
        assert_eq!(screen(&app), Screen::Menu);
        // The title, one line per level, and the key help.
        assert_eq!(drawn_count(&mut app), LEVELS.len() + 2);
    }

    #[test]
    fn the_menu_starts_on_the_first_unsolved_level() {
        let file = scratch_file("first-unsolved");
        let mut progress = Progress::default();
        progress.mark_solved(LEVELS[0].id);
        progress.save(&file).expect("saving should work");

        let app = headless_game(file);
        assert_eq!(selected(&app), 1);
    }

    #[test]
    fn the_menu_starts_at_the_top_when_everything_is_solved() {
        let mut progress = Progress::default();
        for level in LEVELS {
            progress.mark_solved(level.id);
        }
        assert_eq!(first_unsolved(&progress), 0);
    }

    #[test]
    fn down_selects_the_next_level() {
        let mut app = headless_game(scratch_file("down"));
        tap(&mut app, KeyCode::ArrowDown);
        assert_eq!(selected(&app), 1);
    }

    #[test]
    fn the_selection_stops_at_the_top_of_the_list() {
        let mut app = headless_game(scratch_file("top"));
        tap(&mut app, KeyCode::ArrowUp);
        assert_eq!(selected(&app), 0);
    }

    #[test]
    fn the_selection_stops_at_the_bottom_of_the_list() {
        let mut app = headless_game(scratch_file("bottom"));
        for _ in 0..LEVELS.len() + 1 {
            tap(&mut app, KeyCode::ArrowDown);
        }
        assert_eq!(selected(&app), LEVELS.len() - 1);
    }

    #[test]
    fn enter_starts_the_selected_level() {
        let mut app = headless_game(scratch_file("enter"));
        tap(&mut app, KeyCode::ArrowDown);
        tap(&mut app, KeyCode::Enter);

        assert_eq!(screen(&app), Screen::Playing);
        let session = app.world().resource::<Session>();
        assert_eq!(session.board(), &LEVELS[1].board());
    }

    #[test]
    fn pressing_a_key_moves_the_chef() {
        let mut app = headless_game(scratch_file("move"));
        tap(&mut app, KeyCode::Enter);
        let start = LEVELS[0].board().chef();

        tap(&mut app, KeyCode::ArrowDown);

        let chef = app.world().resource::<Session>().board().chef();
        assert_eq!(chef, Pos::new(start.x, start.y + 1));
    }

    #[test]
    fn redrawing_replaces_the_old_picture() {
        let mut app = headless_game(scratch_file("redraw"));
        tap(&mut app, KeyCode::Enter);
        let before = drawn_count(&mut app);

        tap(&mut app, KeyCode::ArrowDown);

        assert_eq!(drawn_count(&mut app), before);
    }

    #[test]
    fn escape_goes_back_to_the_menu() {
        let mut app = headless_game(scratch_file("escape"));
        tap(&mut app, KeyCode::Enter);
        tap(&mut app, KeyCode::Escape);

        assert_eq!(screen(&app), Screen::Menu);
        assert_eq!(drawn_count(&mut app), LEVELS.len() + 2);
    }

    #[test]
    fn solving_a_level_saves_it_to_the_progress_file() {
        let file = scratch_file("solve-saves");
        let mut app = headless_game(file.clone());
        tap(&mut app, KeyCode::Enter);
        for key in FIRST_LEVEL_SOLUTION {
            tap(&mut app, key);
        }

        let saved = Progress::load(&file).expect("loading should work");
        assert!(saved.is_solved(LEVELS[0].id));
    }

    #[test]
    fn enter_on_a_solved_level_starts_the_next_one() {
        let mut app = headless_game(scratch_file("next"));
        tap(&mut app, KeyCode::Enter);
        for key in FIRST_LEVEL_SOLUTION {
            tap(&mut app, key);
        }
        tap(&mut app, KeyCode::Enter);

        assert_eq!(selected(&app), 1);
        let session = app.world().resource::<Session>();
        assert_eq!(session.board(), &LEVELS[1].board());
    }

    #[test]
    fn enter_on_an_unsolved_level_does_nothing() {
        let mut app = headless_game(scratch_file("not-yet"));
        tap(&mut app, KeyCode::Enter);
        tap(&mut app, KeyCode::Enter);

        assert_eq!(screen(&app), Screen::Playing);
        assert_eq!(selected(&app), 0);
    }
}
