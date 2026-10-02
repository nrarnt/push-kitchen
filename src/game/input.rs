use bevy::prelude::*;

use super::levels::LEVELS;
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

/// The command for the key that went down this frame, if any.
/// A key that is held down only counts once.
fn command(keys: &ButtonInput<KeyCode>) -> Option<Command> {
    if keys.any_just_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) {
        Some(Command::Move(Dir::Up))
    } else if keys.any_just_pressed([KeyCode::ArrowDown, KeyCode::KeyS]) {
        Some(Command::Move(Dir::Down))
    } else if keys.any_just_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
        Some(Command::Move(Dir::Left))
    } else if keys.any_just_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) {
        Some(Command::Move(Dir::Right))
    } else if keys.any_just_pressed([KeyCode::KeyZ, KeyCode::KeyY]) {
        // Both, so the key labelled Z works on US and Swiss keyboards.
        Some(Command::Undo)
    } else if keys.just_pressed(KeyCode::KeyR) {
        Some(Command::Restart)
    } else if keys.any_just_pressed([KeyCode::Enter, KeyCode::Space]) {
        Some(Command::Next)
    } else if keys.just_pressed(KeyCode::Escape) {
        Some(Command::Menu)
    } else {
        None
    }
}

pub fn handle_input(
    mut commands: Commands,
    assets: Res<AssetServer>,
    keys: Res<ButtonInput<KeyCode>>,
    mut session: ResMut<Session>,
    mut selected: ResMut<Selected>,
    mut screen: ResMut<NextState<Screen>>,
) {
    match command(&keys) {
        Some(Command::Move(dir)) => {
            session.step(dir);
            let sfx = match session.previous() {
                Some(before) => sound_of_move(before, session.board()),
                // Nothing changed: the chef walked into something solid.
                None => Sfx::Bump,
            };
            sound::play(&mut commands, &assets, sfx);
        }
        Some(Command::Undo) => session.undo(),
        Some(Command::Restart) => session.restart(),
        Some(Command::Next) if session.board().is_solved() => {
            if selected.0 + 1 < LEVELS.len() {
                // Entering `Playing` again starts the newly selected level.
                selected.0 += 1;
                screen.set(Screen::Playing);
            } else {
                screen.set(Screen::Menu);
            }
        }
        Some(Command::Menu) => screen.set(Screen::Menu),
        Some(Command::Next) | None => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pressing(key: KeyCode) -> ButtonInput<KeyCode> {
        let mut keys = ButtonInput::default();
        keys.press(key);
        keys
    }

    #[test]
    fn arrow_keys_move_the_chef() {
        assert_eq!(command(&pressing(KeyCode::ArrowUp)), Some(Command::Move(Dir::Up)));
        assert_eq!(command(&pressing(KeyCode::ArrowDown)), Some(Command::Move(Dir::Down)));
        assert_eq!(command(&pressing(KeyCode::ArrowLeft)), Some(Command::Move(Dir::Left)));
        assert_eq!(command(&pressing(KeyCode::ArrowRight)), Some(Command::Move(Dir::Right)));
    }

    #[test]
    fn wasd_keys_move_the_chef() {
        assert_eq!(command(&pressing(KeyCode::KeyW)), Some(Command::Move(Dir::Up)));
        assert_eq!(command(&pressing(KeyCode::KeyS)), Some(Command::Move(Dir::Down)));
        assert_eq!(command(&pressing(KeyCode::KeyA)), Some(Command::Move(Dir::Left)));
        assert_eq!(command(&pressing(KeyCode::KeyD)), Some(Command::Move(Dir::Right)));
    }

    #[test]
    fn z_undoes() {
        assert_eq!(command(&pressing(KeyCode::KeyZ)), Some(Command::Undo));
    }

    #[test]
    fn z_undoes_on_a_swiss_keyboard_too() {
        // `KeyCode` names the key's place on a US keyboard. A Swiss keyboard
        // has its Z where the US one has Y.
        assert_eq!(command(&pressing(KeyCode::KeyY)), Some(Command::Undo));
    }

    #[test]
    fn r_restarts() {
        assert_eq!(command(&pressing(KeyCode::KeyR)), Some(Command::Restart));
    }

    #[test]
    fn enter_and_space_ask_for_the_next_kitchen() {
        assert_eq!(command(&pressing(KeyCode::Enter)), Some(Command::Next));
        assert_eq!(command(&pressing(KeyCode::Space)), Some(Command::Next));
    }

    #[test]
    fn escape_goes_to_the_menu() {
        assert_eq!(command(&pressing(KeyCode::Escape)), Some(Command::Menu));
    }

    #[test]
    fn no_key_means_no_command() {
        assert_eq!(command(&ButtonInput::default()), None);
    }

    #[test]
    fn a_held_key_does_not_repeat() {
        let mut keys = pressing(KeyCode::ArrowUp);
        // What Bevy does between frames: the key stays pressed, but is no
        // longer "just" pressed.
        keys.clear();
        assert_eq!(command(&keys), None);
    }
}
