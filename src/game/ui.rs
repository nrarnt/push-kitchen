use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;

use super::input::keys_pressed;
use super::levels::LEVELS;
use super::progress::Progress;
use super::view::{self, Drawn};
use super::{Screen, Selected};

const TITLE_Y: f32 = 300.0;
const FIRST_LINE_Y: f32 = 225.0;
const LINE_STEP: f32 = 34.0;
const HELP_Y: f32 = -325.0;

/// How high on the screen the menu line of level number `i` goes.
fn line_y(i: usize) -> f32 {
    FIRST_LINE_Y - i as f32 * LINE_STEP
}

/// What the player asked for in the menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MenuCommand {
    Up,
    Down,
    Play,
}

/// The menu command a key stands for, if any.
fn menu_command(key: KeyCode) -> Option<MenuCommand> {
    match key {
        KeyCode::ArrowUp | KeyCode::KeyW => Some(MenuCommand::Up),
        KeyCode::ArrowDown | KeyCode::KeyS => Some(MenuCommand::Down),
        KeyCode::Enter | KeyCode::Space => Some(MenuCommand::Play),
        _ => None,
    }
}

pub fn handle_menu_input(
    mut keys: MessageReader<KeyboardInput>,
    mut selected: ResMut<Selected>,
    mut screen: ResMut<NextState<Screen>>,
) {
    for key in keys_pressed(&mut keys) {
        match menu_command(key) {
            // The selection stops at both ends of the list.
            Some(MenuCommand::Up) if selected.0 > 0 => selected.0 -= 1,
            Some(MenuCommand::Down) if selected.0 + 1 < LEVELS.len() => selected.0 += 1,
            Some(MenuCommand::Play) => screen.set(Screen::Playing),
            _ => {}
        }
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

    commands.spawn((Drawn, view::caption("Push Kitchen", 48.0, view::LIGHT_TEXT, TITLE_Y)));

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
        commands.spawn((Drawn, view::caption(&text, 24.0, colour, line_y(i))));
    }

    let keys = "Up / Down: choose    Enter: play";
    commands.spawn((Drawn, view::caption(keys, 20.0, view::DIM_TEXT, HELP_Y)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn up_and_down_move_the_selection() {
        assert_eq!(menu_command(KeyCode::ArrowUp), Some(MenuCommand::Up));
        assert_eq!(menu_command(KeyCode::KeyW), Some(MenuCommand::Up));
        assert_eq!(menu_command(KeyCode::ArrowDown), Some(MenuCommand::Down));
        assert_eq!(menu_command(KeyCode::KeyS), Some(MenuCommand::Down));
    }

    #[test]
    fn enter_and_space_play_the_selected_level() {
        assert_eq!(menu_command(KeyCode::Enter), Some(MenuCommand::Play));
        assert_eq!(menu_command(KeyCode::Space), Some(MenuCommand::Play));
    }

    #[test]
    fn the_whole_menu_fits_in_the_view() {
        let top = view::VIEW_HEIGHT / 2.0;
        // Half a line of text above and below each position.
        assert!(TITLE_Y + 30.0 <= top);
        assert!(line_y(LEVELS.len() - 1) - 30.0 >= HELP_Y);
        assert!(HELP_Y - 15.0 >= -top);
    }

    #[test]
    fn other_keys_do_nothing_in_the_menu() {
        assert_eq!(menu_command(KeyCode::KeyZ), None);
    }
}
