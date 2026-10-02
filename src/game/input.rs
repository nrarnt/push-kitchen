use bevy::input::keyboard::KeyboardInput;
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

/// The keys that went down since the last frame, in the order they were
/// pressed. Reading key presses one by one, rather than asking "is this key
/// down?", means none is lost when several arrive in the same frame. A key
/// that is held down counts once.
pub fn keys_pressed(keys: &mut MessageReader<KeyboardInput>) -> impl Iterator<Item = KeyCode> {
    keys.read()
        .filter(|key| key.state.is_pressed() && !key.repeat)
        .map(|key| key.key_code)
}

pub fn handle_input(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut keys: MessageReader<KeyboardInput>,
    mut session: ResMut<Session>,
    mut selected: ResMut<Selected>,
    mut screen: ResMut<NextState<Screen>>,
) {
    for key in keys_pressed(&mut keys) {
        match command(key) {
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

    #[test]
    fn other_keys_mean_no_command() {
        assert_eq!(command(KeyCode::KeyQ), None);
    }
}
