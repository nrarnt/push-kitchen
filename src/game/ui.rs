use bevy::prelude::*;

use super::levels::LEVELS;
use super::progress::Progress;
use super::view::{self, Drawn};
use super::{Screen, Selected};

/// What the player asked for in the menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MenuCommand {
    Up,
    Down,
    Play,
}

fn menu_command(keys: &ButtonInput<KeyCode>) -> Option<MenuCommand> {
    if keys.any_just_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) {
        Some(MenuCommand::Up)
    } else if keys.any_just_pressed([KeyCode::ArrowDown, KeyCode::KeyS]) {
        Some(MenuCommand::Down)
    } else if keys.any_just_pressed([KeyCode::Enter, KeyCode::Space]) {
        Some(MenuCommand::Play)
    } else {
        None
    }
}

pub fn handle_menu_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut selected: ResMut<Selected>,
    mut screen: ResMut<NextState<Screen>>,
) {
    match menu_command(&keys) {
        // The selection stops at both ends of the list.
        Some(MenuCommand::Up) if selected.0 > 0 => selected.0 -= 1,
        Some(MenuCommand::Down) if selected.0 + 1 < LEVELS.len() => selected.0 += 1,
        Some(MenuCommand::Play) => screen.set(Screen::Playing),
        _ => {}
    }
}

/// Throws away the old picture and draws the list of levels.
pub fn draw_menu(
    mut commands: Commands,
    selected: Res<Selected>,
    progress: Res<Progress>,
    drawn: Query<Entity, With<Drawn>>,
) {
    view::clear(&mut commands, &drawn);

    commands.spawn((Drawn, view::caption("Push Kitchen", 56.0, view::LIGHT_TEXT, 220.0)));

    for (i, level) in LEVELS.iter().enumerate() {
        let mut text = format!("{}. {}", i + 1, level.name);
        if progress.is_solved(level.id) {
            text += " (served)";
        }
        let (text, colour) = if i == selected.0 {
            (format!("> {text} <"), view::LIGHT_TEXT)
        } else {
            (text, view::DIM_TEXT)
        };
        let y = 110.0 - i as f32 * 44.0;
        commands.spawn((Drawn, view::caption(&text, 28.0, colour, y)));
    }

    let keys = "Up / Down: choose    Enter: play";
    commands.spawn((Drawn, view::caption(keys, 20.0, view::DIM_TEXT, -300.0)));
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
    fn up_and_down_move_the_selection() {
        assert_eq!(menu_command(&pressing(KeyCode::ArrowUp)), Some(MenuCommand::Up));
        assert_eq!(menu_command(&pressing(KeyCode::KeyW)), Some(MenuCommand::Up));
        assert_eq!(menu_command(&pressing(KeyCode::ArrowDown)), Some(MenuCommand::Down));
        assert_eq!(menu_command(&pressing(KeyCode::KeyS)), Some(MenuCommand::Down));
    }

    #[test]
    fn enter_and_space_play_the_selected_level() {
        assert_eq!(menu_command(&pressing(KeyCode::Enter)), Some(MenuCommand::Play));
        assert_eq!(menu_command(&pressing(KeyCode::Space)), Some(MenuCommand::Play));
    }

    #[test]
    fn other_keys_do_nothing_in_the_menu() {
        assert_eq!(menu_command(&pressing(KeyCode::KeyZ)), None);
    }
}
