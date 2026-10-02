use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;

use super::layout::{BoardLayout, ViewSize};
use super::levels::LEVELS;
use super::pointer::{Button, Gesture, TouchMode};
use super::session::Session;
use super::sound::{self, Sfx, sound_of_move};
use super::{Screen, Selected};
use crate::puzzle::{Board, Dir};

/// What the player asked for with a key press, a tap or a swipe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Command {
    /// Move the chef this way as seen on screen. In a kitchen that is drawn
    /// turned, that is a different direction in the kitchen itself.
    Move(Dir),
    Undo,
    Restart,
    /// On to the next kitchen. Only does something once this one is solved.
    Next,
    Menu,
}

/// How the kitchen on screen is doing, which decides what the player can ask for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Cooking,
    Solved,
    /// The chef stepped on something hot. All that is left is to start again.
    Burnt,
}

impl Stage {
    pub fn of(board: &Board) -> Stage {
        if board.is_burnt() {
            Stage::Burnt
        } else if board.is_solved() {
            Stage::Solved
        } else {
            Stage::Cooking
        }
    }
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

/// The buttons shown under the kitchen in touch mode, each with what it
/// does. Once the kitchen is solved, "Restart" gives way to "Next". In a
/// burnt kitchen there is nothing to undo.
fn buttons_and_commands(stage: Stage, layout: &BoardLayout) -> Vec<(Button, Command)> {
    match stage {
        Stage::Cooking => {
            let [undo, restart, menu] = layout.buttons(["Undo", "Restart", "Menu"]);
            vec![(undo, Command::Undo), (restart, Command::Restart), (menu, Command::Menu)]
        }
        Stage::Solved => {
            let [undo, next, menu] = layout.buttons(["Undo", "Next", "Menu"]);
            vec![(undo, Command::Undo), (next, Command::Next), (menu, Command::Menu)]
        }
        Stage::Burnt => {
            let [restart, menu] = layout.buttons(["Restart", "Menu"]);
            vec![(restart, Command::Restart), (menu, Command::Menu)]
        }
    }
}

/// The touch buttons to draw under the kitchen.
pub fn buttons(stage: Stage, layout: &BoardLayout) -> impl Iterator<Item = Button> {
    buttons_and_commands(stage, layout)
        .into_iter()
        .map(|(button, _)| button)
}

/// The command a tap or swipe stands for while playing, if any. A swipe
/// moves the chef; a tap only means something on a button, and only while
/// the buttons are shown.
fn gesture_command(
    gesture: Gesture,
    buttons_shown: bool,
    stage: Stage,
    layout: &BoardLayout,
) -> Option<Command> {
    match gesture {
        Gesture::Swipe(dir) => Some(Command::Move(dir)),
        Gesture::Tap(at) if buttons_shown => buttons_and_commands(stage, layout)
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
    view: Res<ViewSize>,
    mut session: ResMut<Session>,
    mut selected: ResMut<Selected>,
    mut screen: ResMut<NextState<Screen>>,
) {
    // Everything asked for since the last frame: by keyboard, then by touch.
    // Taps are judged against the buttons as they were drawn.
    let stage = Stage::of(session.board());
    let layout = BoardLayout::new(view.0, session.board().width(), session.board().height());
    let mut asked: Vec<Command> = keys_pressed(&mut keys).filter_map(command).collect();
    asked.extend(
        gestures
            .read()
            .filter_map(|gesture| gesture_command(*gesture, touch.0, stage, &layout)),
    );

    for command in asked {
        match command {
            Command::Move(on_screen) => {
                session.step(layout.turn(on_screen));
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
    use crate::puzzle::parse;

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

    /// A kitchen 9 wide and 6 high on a phone held upright.
    fn layout() -> BoardLayout {
        BoardLayout::new(Vec2::new(430.0, 900.0), 9, 6)
    }

    /// Where the button with this label is drawn.
    fn button_centre(label: &str, stage: Stage) -> Vec2 {
        buttons(stage, &layout())
            .find(|button| button.label == label)
            .expect("there should be a button with that label")
            .centre
    }

    #[test]
    fn a_swipe_moves_the_chef() {
        let swipe = Gesture::Swipe(Dir::Left);
        let command = gesture_command(swipe, false, Stage::Cooking, &layout());
        assert_eq!(command, Some(Command::Move(Dir::Left)));
    }

    #[test]
    fn tapping_a_button_does_what_it_says() {
        let tapped = |label| {
            let tap = Gesture::Tap(button_centre(label, Stage::Cooking));
            gesture_command(tap, true, Stage::Cooking, &layout())
        };
        assert_eq!(tapped("Undo"), Some(Command::Undo));
        assert_eq!(tapped("Restart"), Some(Command::Restart));
        assert_eq!(tapped("Menu"), Some(Command::Menu));
    }

    #[test]
    fn a_solved_kitchen_offers_next_instead_of_restart() {
        let tap = Gesture::Tap(button_centre("Next", Stage::Solved));
        assert_eq!(gesture_command(tap, true, Stage::Solved, &layout()), Some(Command::Next));
    }

    #[test]
    fn a_burnt_kitchen_offers_only_restart_and_menu() {
        let labels: Vec<&str> = buttons(Stage::Burnt, &layout()).map(|button| button.label).collect();
        assert_eq!(labels, ["Restart", "Menu"]);
    }

    #[test]
    fn tapping_restart_in_a_burnt_kitchen_restarts_it() {
        let tap = Gesture::Tap(button_centre("Restart", Stage::Burnt));
        assert_eq!(gesture_command(tap, true, Stage::Burnt, &layout()), Some(Command::Restart));
    }

    #[test]
    fn the_stage_follows_the_board() {
        let cooking = parse("#@tT#\n#~###").unwrap();
        assert_eq!(Stage::of(&cooking), Stage::Cooking);
        let solved = cooking.step(Dir::Right).unwrap();
        assert_eq!(Stage::of(&solved), Stage::Solved);
        let burnt = cooking.step(Dir::Down).unwrap();
        assert_eq!(Stage::of(&burnt), Stage::Burnt);
    }

    #[test]
    fn a_tap_beside_the_buttons_does_nothing() {
        let tap = Gesture::Tap(Vec2::ZERO);
        assert_eq!(gesture_command(tap, true, Stage::Cooking, &layout()), None);
    }

    #[test]
    fn buttons_that_are_not_shown_cannot_be_tapped() {
        let tap = Gesture::Tap(button_centre("Undo", Stage::Cooking));
        assert_eq!(gesture_command(tap, false, Stage::Cooking, &layout()), None);
    }

    #[test]
    fn other_keys_mean_no_command() {
        assert_eq!(command(KeyCode::KeyQ), None);
    }
}
