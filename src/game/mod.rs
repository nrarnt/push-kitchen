//! The Bevy layer: shows the puzzle on screen and feeds it key presses.

mod input;
mod levels;
mod progress;
mod session;
mod sound;
mod ui;
mod view;

use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;

use levels::LEVELS;
use progress::Progress;
use session::Session;

pub use progress::SaveSlot;

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

/// Holds on to every picture and sound for as long as the game runs. Bevy
/// drops a file from memory once nothing uses it, and would otherwise fetch
/// the sprites again each time a level starts.
#[derive(Resource)]
struct Preloaded {
    _pictures: Vec<Handle<Image>>,
    _sounds: Vec<Handle<AudioSource>>,
}

fn preload_assets(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(Preloaded {
        _pictures: view::preload(&assets),
        _sounds: sound::preload(&assets),
    });
}

/// Throws away key presses nobody has acted on yet. Run whenever the screen
/// changes, so that a key meant for the old screen is not read again by the
/// new one.
fn forget_keys(mut keys: ResMut<Messages<KeyboardInput>>) {
    keys.clear();
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
    slot: Res<SaveSlot>,
) {
    if session.board().is_solved()
        && progress.mark_solved(LEVELS[selected.0].id)
        && let Err(error) = progress.save(&slot)
    {
        // Not being able to save should not stop the game.
        warn!("could not save progress to {:?}: {error}", *slot);
    }
}

pub struct GamePlugin {
    pub save_slot: SaveSlot,
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        let progress = Progress::load(&self.save_slot).unwrap_or_else(|error| {
            warn!("could not read progress from {:?}: {error}", self.save_slot);
            Progress::default()
        });

        app.init_state::<Screen>()
            .insert_resource(ClearColor(view::BACKGROUND))
            .insert_resource(Selected(first_unsolved(&progress)))
            .insert_resource(progress)
            .insert_resource(self.save_slot.clone())
            .add_systems(Startup, (view::spawn_camera, preload_assets))
            .add_systems(OnEnter(Screen::Menu), (forget_keys, ui::draw_menu))
            .add_systems(OnEnter(Screen::Playing), (forget_keys, start_level))
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
                        view::slide,
                    )
                        .chain()
                        .run_if(in_state(Screen::Playing)),
                ),
            );
    }
}

#[cfg(test)]
mod tests {
    use bevy::input::ButtonState;
    use bevy::input::keyboard::{Key, NativeKey};
    use bevy::state::app::StatesPlugin;

    use super::*;
    use crate::puzzle::Pos;
    use progress::scratch_slot;

    /// The keys that solve the first level.
    const FIRST_LEVEL_SOLUTION: [KeyCode; 5] = [
        KeyCode::ArrowDown,
        KeyCode::ArrowRight,
        KeyCode::ArrowRight,
        KeyCode::ArrowUp,
        KeyCode::ArrowUp,
    ];

    /// The game without a window: just enough of Bevy to run our systems.
    fn headless_game(save_slot: SaveSlot) -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin, AssetPlugin::default()))
            .init_asset::<Image>()
            .init_asset::<AudioSource>()
            .add_message::<KeyboardInput>()
            .add_plugins(GamePlugin { save_slot });
        app.update();
        app
    }

    /// Tells the game a key went down, as the keyboard would. `repeat` is
    /// what a keyboard sends over and over while a key is held.
    fn press(app: &mut App, key_code: KeyCode, repeat: bool) {
        app.world_mut().write_message(KeyboardInput {
            key_code,
            logical_key: Key::Unidentified(NativeKey::Unidentified),
            state: ButtonState::Pressed,
            text: None,
            repeat,
            window: Entity::PLACEHOLDER,
        });
    }

    /// Presses a key, then gives the game two frames: one to act on it, and
    /// one for a change of screen to take effect.
    fn tap(app: &mut App, key: KeyCode) {
        press(app, key, false);
        app.update();
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
        let mut app = headless_game(scratch_slot("starts-in-menu"));
        assert_eq!(screen(&app), Screen::Menu);
        // The title, one line per level, and the key help.
        assert_eq!(drawn_count(&mut app), LEVELS.len() + 2);
    }

    #[test]
    fn the_menu_starts_on_the_first_unsolved_level() {
        let slot = scratch_slot("first-unsolved");
        let mut progress = Progress::default();
        progress.mark_solved(LEVELS[0].id);
        progress.save(&slot).expect("saving should work");

        let app = headless_game(slot);
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
        let mut app = headless_game(scratch_slot("down"));
        tap(&mut app, KeyCode::ArrowDown);
        assert_eq!(selected(&app), 1);
    }

    #[test]
    fn the_selection_stops_at_the_top_of_the_list() {
        let mut app = headless_game(scratch_slot("top"));
        tap(&mut app, KeyCode::ArrowUp);
        assert_eq!(selected(&app), 0);
    }

    #[test]
    fn the_selection_stops_at_the_bottom_of_the_list() {
        let mut app = headless_game(scratch_slot("bottom"));
        for _ in 0..LEVELS.len() + 1 {
            tap(&mut app, KeyCode::ArrowDown);
        }
        assert_eq!(selected(&app), LEVELS.len() - 1);
    }

    #[test]
    fn enter_starts_the_selected_level() {
        let mut app = headless_game(scratch_slot("enter"));
        tap(&mut app, KeyCode::ArrowDown);
        tap(&mut app, KeyCode::Enter);

        assert_eq!(screen(&app), Screen::Playing);
        let session = app.world().resource::<Session>();
        assert_eq!(session.board(), &LEVELS[1].board());
    }

    #[test]
    fn pressing_a_key_moves_the_chef() {
        let mut app = headless_game(scratch_slot("move"));
        tap(&mut app, KeyCode::Enter);
        let start = LEVELS[0].board().chef();

        tap(&mut app, KeyCode::ArrowDown);

        let chef = app.world().resource::<Session>().board().chef();
        assert_eq!(chef, Pos::new(start.x, start.y + 1));
    }

    #[test]
    fn keys_pressed_in_the_same_frame_are_all_handled_in_order() {
        let mut app = headless_game(scratch_slot("same-frame"));
        tap(&mut app, KeyCode::Enter);
        let start = LEVELS[0].board().chef();

        press(&mut app, KeyCode::ArrowDown, false);
        press(&mut app, KeyCode::ArrowRight, false);
        app.update();

        let chef = app.world().resource::<Session>().board().chef();
        assert_eq!(chef, Pos::new(start.x + 1, start.y + 1));
    }

    #[test]
    fn menu_keys_pressed_in_the_same_frame_are_all_handled() {
        let mut app = headless_game(scratch_slot("menu-same-frame"));
        press(&mut app, KeyCode::ArrowDown, false);
        press(&mut app, KeyCode::ArrowDown, false);
        app.update();
        assert_eq!(selected(&app), 2);
    }

    #[test]
    fn a_key_pressed_in_the_menu_is_not_acted_on_again_by_the_level() {
        let mut app = headless_game(scratch_slot("menu-key-leak"));
        // Up does nothing at the top of the menu, but would move the chef.
        press(&mut app, KeyCode::ArrowUp, false);
        press(&mut app, KeyCode::Enter, false);
        app.update();
        app.update();

        let chef = app.world().resource::<Session>().board().chef();
        assert_eq!(chef, LEVELS[0].board().chef());
    }

    #[test]
    fn a_key_pressed_in_a_level_is_not_acted_on_again_by_the_menu() {
        let mut app = headless_game(scratch_slot("level-key-leak"));
        tap(&mut app, KeyCode::Enter);
        press(&mut app, KeyCode::ArrowDown, false);
        press(&mut app, KeyCode::Escape, false);
        app.update();
        app.update();

        assert_eq!(screen(&app), Screen::Menu);
        assert_eq!(selected(&app), 0);
    }

    #[test]
    fn a_held_key_does_not_repeat() {
        let mut app = headless_game(scratch_slot("held"));
        tap(&mut app, KeyCode::Enter);
        let start = LEVELS[0].board().chef();

        press(&mut app, KeyCode::ArrowDown, true);
        app.update();

        let chef = app.world().resource::<Session>().board().chef();
        assert_eq!(chef, start);
    }

    #[test]
    fn redrawing_replaces_the_old_picture() {
        let mut app = headless_game(scratch_slot("redraw"));
        tap(&mut app, KeyCode::Enter);
        let before = drawn_count(&mut app);

        tap(&mut app, KeyCode::ArrowDown);

        assert_eq!(drawn_count(&mut app), before);
    }

    fn count<C: Component>(app: &mut App) -> usize {
        app.world_mut().query::<&C>().iter(app.world()).count()
    }

    #[test]
    fn nothing_slides_when_a_level_starts() {
        let mut app = headless_game(scratch_slot("slide-none"));
        tap(&mut app, KeyCode::Enter);
        assert_eq!(count::<view::Slide>(&mut app), 0);
    }

    #[test]
    fn walking_slides_the_chef() {
        let mut app = headless_game(scratch_slot("slide-chef"));
        tap(&mut app, KeyCode::Enter);
        tap(&mut app, KeyCode::ArrowDown);
        assert_eq!(count::<view::Slide>(&mut app), 1);
    }

    #[test]
    fn pushing_slides_the_chef_and_the_item() {
        let mut app = headless_game(scratch_slot("slide-push"));
        tap(&mut app, KeyCode::Enter);
        // All but the last key of the solution: the last of these is a push.
        for key in &FIRST_LEVEL_SOLUTION[..4] {
            tap(&mut app, *key);
        }
        assert_eq!(count::<view::Slide>(&mut app), 2);
    }

    #[test]
    fn a_move_plays_one_sound() {
        let mut app = headless_game(scratch_slot("sound"));
        tap(&mut app, KeyCode::Enter);
        tap(&mut app, KeyCode::ArrowDown);
        assert_eq!(count::<AudioPlayer>(&mut app), 1);
    }

    #[test]
    fn escape_goes_back_to_the_menu() {
        let mut app = headless_game(scratch_slot("escape"));
        tap(&mut app, KeyCode::Enter);
        tap(&mut app, KeyCode::Escape);

        assert_eq!(screen(&app), Screen::Menu);
        assert_eq!(drawn_count(&mut app), LEVELS.len() + 2);
    }

    #[test]
    fn solving_a_level_saves_it() {
        let slot = scratch_slot("solve-saves");
        let mut app = headless_game(slot.clone());
        tap(&mut app, KeyCode::Enter);
        for key in FIRST_LEVEL_SOLUTION {
            tap(&mut app, key);
        }

        let saved = Progress::load(&slot).expect("loading should work");
        assert!(saved.is_solved(LEVELS[0].id));
    }

    #[test]
    fn enter_on_a_solved_level_starts_the_next_one() {
        let mut app = headless_game(scratch_slot("next"));
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
        let mut app = headless_game(scratch_slot("not-yet"));
        tap(&mut app, KeyCode::Enter);
        tap(&mut app, KeyCode::Enter);

        assert_eq!(screen(&app), Screen::Playing);
        assert_eq!(selected(&app), 0);
    }
}
