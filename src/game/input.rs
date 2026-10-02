use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;

use super::levels::LEVELS;
use super::pointer::{Button, Gesture, TouchMode};
use super::session::Session;
use super::sound::{self, Sfx, sound_of_move};
use super::{Screen, Selected};
use crate::puzzle::Dir;

/// What the player asked for with a key press.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Command {
    Move(Dir),
    Undo,
    Restart,
    /// On to the next kitchen. Only does something once this one is solved.
    Next,
    Menu,
}

/// The command a key stands for while playing, if any.
fn command(key: KeyCode) -> Option<Command> {
    match key {
        KeyCode::ArrowUp | KeyCode::KeyW => Some(Command::Move(Dir::Up)),
        KeyCode::ArrowDown | KeyCode::KeyS => Some(Command::Move(Dir::Down)),
        KeyCode::ArrowLeft | KeyCode::KeyA => Some(Command::Move(Dir::Left)),
        KeyCode::ArrowRight | KeyCode::KeyD => Some(Command::Move(Dir::Right)),
        // Both, so the key labelled Z works on US and Swiss keyboards.
        KeyCode::KeyZ | KeyCode::KeyY => Some(Command::Undo),
        KeyCode::KeyR => Some(Command::Restart),
        KeyCode::Enter | KeyCode::Space => Some(Command::Next),
        KeyCode::Escape => Some(Command::Menu),
        _ => None,
    }
}

/// How high in the view the row of touch buttons sits.
const BUTTON_Y: f32 = -310.0;

/// The buttons shown under the kitchen in touch mode, each with what it
/// does. Once the kitchen is solved, "Restart" gives way to "Next".
fn buttons_and_commands(solved: bool) -> [(Button, Command); 3] {
    let button = |label, x| Button { label, centre: Vec2::new(x, BUTTON_Y) };
    let middle = if solved {
        (button("Next", 0.0), Command::Next)
    } else {
        (button("Restart", 0.0), Command::Restart)
    };
    [
        (button("Undo", -250.0), Command::Undo),
        middle,
        (button("Menu", 250.0), Command::Menu),
    ]
}

/// The touch buttons to draw under the kitchen.
pub fn buttons(solved: bool) -> impl Iterator<Item = Button> {
    buttons_and_commands(solved).into_iter().map(|(button, _)| button)
}

/// The command a tap or swipe stands for while playing, if any. A swipe
/// moves the chef; a tap only means something on a button, and only while
/// the buttons are shown.
fn gesture_command(gesture: Gesture, buttons_shown: bool, solved: bool) -> Option<Command> {
    match gesture {
        Gesture::Swipe(dir) => Some(Command::Move(dir)),
        Gesture::Tap(at) if buttons_shown => buttons_and_commands(solved)
            .into_iter()
            .find(|(button, _)| button.contains(at))
            .map(|(_, command)| command),
        Gesture::Tap(_) => None,
    }
}

/// The keys that went down since the last frame, in the order they were
/// pressed. Reading key presses one by one, rather than asking "is this key
/// down?", means none is lost when several arrive in the same frame. A key
/// that is held down counts once.
pub fn keys_pressed(keys: &mut MessageReader<KeyboardInput>) -> impl Iterator<Item = KeyCode> {
    keys.read()
        .filter(|key| key.state.is_pressed() && !key.repeat)
        .map(|key| key.key_code)
}

// A Bevy system takes one argument for each thing it reads or changes, and
// this one touches a lot.
#[allow(clippy::too_many_arguments)]
pub fn handle_input(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut keys: MessageReader<KeyboardInput>,
    mut gestures: MessageReader<Gesture>,
    touch: Res<TouchMode>,
    mut session: ResMut<Session>,
    mut selected: ResMut<Selected>,
    mut screen: ResMut<NextState<Screen>>,
) {
    // Everything asked for since the last frame: by keyboard, then by touch.
    // Taps are judged against the buttons as they were drawn.
    let solved = session.board().is_solved();
    let mut asked: Vec<Command> = keys_pressed(&mut keys).filter_map(command).collect();
    asked.extend(
        gestures
            .read()
            .filter_map(|gesture| gesture_command(*gesture, touch.0, solved)),
    );

    for command in asked {
        match command {
            Command::Move(dir) => {
                session.step(dir);
                let sfx = match session.previous() {
                    Some(before) => sound_of_move(before, session.board()),
                    // Nothing changed: the chef walked into something solid.
                    None => Sfx::Bump,
                };
                sound::play(&mut commands, &assets, sfx);
            }
            Command::Undo => session.undo(),
            Command::Restart => session.restart(),
            Command::Next if session.board().is_solved() => {
                if selected.0 + 1 < LEVELS.len() {
                    // Entering `Playing` again starts the newly selected level.
                    selected.0 += 1;
                    screen.set(Screen::Playing);
                } else {
                    screen.set(Screen::Menu);
                }
            }
            Command::Menu => screen.set(Screen::Menu),
            Command::Next => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arrow_keys_move_the_chef() {
        assert_eq!(command(KeyCode::ArrowUp), Some(Command::Move(Dir::Up)));
        assert_eq!(command(KeyCode::ArrowDown), Some(Command::Move(Dir::Down)));
        assert_eq!(command(KeyCode::ArrowLeft), Some(Command::Move(Dir::Left)));
        assert_eq!(command(KeyCode::ArrowRight), Some(Command::Move(Dir::Right)));
    }

    #[test]
    fn wasd_keys_move_the_chef() {
        assert_eq!(command(KeyCode::KeyW), Some(Command::Move(Dir::Up)));
        assert_eq!(command(KeyCode::KeyS), Some(Command::Move(Dir::Down)));
        assert_eq!(command(KeyCode::KeyA), Some(Command::Move(Dir::Left)));
        assert_eq!(command(KeyCode::KeyD), Some(Command::Move(Dir::Right)));
    }

    #[test]
    fn z_undoes() {
        assert_eq!(command(KeyCode::KeyZ), Some(Command::Undo));
    }

    #[test]
    fn z_undoes_on_a_swiss_keyboard_too() {
        // `KeyCode` names the key's place on a US keyboard. A Swiss keyboard
        // has its Z where the US one has Y.
        assert_eq!(command(KeyCode::KeyY), Some(Command::Undo));
    }

    #[test]
    fn r_restarts() {
        assert_eq!(command(KeyCode::KeyR), Some(Command::Restart));
    }

    #[test]
    fn enter_and_space_ask_for_the_next_kitchen() {
        assert_eq!(command(KeyCode::Enter), Some(Command::Next));
        assert_eq!(command(KeyCode::Space), Some(Command::Next));
    }

    #[test]
    fn escape_goes_to_the_menu() {
        assert_eq!(command(KeyCode::Escape), Some(Command::Menu));
    }

    /// Where the button with this label is drawn.
    fn button_centre(label: &str, solved: bool) -> Vec2 {
        buttons(solved)
            .find(|button| button.label == label)
            .expect("there should be a button with that label")
            .centre
    }

    #[test]
    fn a_swipe_moves_the_chef() {
        let swipe = Gesture::Swipe(Dir::Left);
        assert_eq!(gesture_command(swipe, false, false), Some(Command::Move(Dir::Left)));
    }

    #[test]
    fn tapping_a_button_does_what_it_says() {
        let tap = |label| Gesture::Tap(button_centre(label, false));
        assert_eq!(gesture_command(tap("Undo"), true, false), Some(Command::Undo));
        assert_eq!(gesture_command(tap("Restart"), true, false), Some(Command::Restart));
        assert_eq!(gesture_command(tap("Menu"), true, false), Some(Command::Menu));
    }

    #[test]
    fn a_solved_kitchen_offers_next_instead_of_restart() {
        let tap = Gesture::Tap(button_centre("Next", true));
        assert_eq!(gesture_command(tap, true, true), Some(Command::Next));
    }

    #[test]
    fn a_tap_beside_the_buttons_does_nothing() {
        assert_eq!(gesture_command(Gesture::Tap(Vec2::ZERO), true, false), None);
    }

    #[test]
    fn buttons_that_are_not_shown_cannot_be_tapped() {
        let tap = Gesture::Tap(button_centre("Undo", false));
        assert_eq!(gesture_command(tap, false, false), None);
    }

    #[test]
    fn the_buttons_fit_in_the_view_side_by_side() {
        let all: Vec<Button> = buttons(false).collect();
        let half = Button::SIZE / 2.0;
        for pair in all.windows(2) {
            assert!(pair[0].centre.x + half.x <= pair[1].centre.x - half.x);
        }
        for button in all {
            assert!(button.centre.x.abs() + half.x <= crate::game::view::VIEW_WIDTH / 2.0);
            assert!(button.centre.y - half.y >= -crate::game::view::VIEW_HEIGHT / 2.0);
        }
    }

    #[test]
    fn other_keys_mean_no_command() {
        assert_eq!(command(KeyCode::KeyQ), None);
    }
}
