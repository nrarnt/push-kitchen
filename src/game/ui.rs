use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;

use super::input::keys_pressed;
use super::levels::LEVELS;
use super::pointer::{Button, Gesture, TouchMode};
use crate::puzzle::Dir;
use super::progress::Progress;
use super::view::{self, Drawn};
use super::{Screen, Selected};

const TITLE_Y: f32 = 300.0;
const FIRST_LINE_Y: f32 = 225.0;
const LINE_STEP: f32 = 34.0;
const HELP_Y: f32 = -325.0;

/// Shown in touch mode: starts the selected kitchen.
const PLAY_BUTTON: Button = Button {
    label: "Play",
    centre: Vec2::new(0.0, -262.0),
};

/// How high on the screen the menu line of level number `i` goes.
pub fn line_y(i: usize) -> f32 {
    FIRST_LINE_Y - i as f32 * LINE_STEP
}

/// The number of the level whose menu line is at height `y`, if any.
fn line_at(y: f32) -> Option<usize> {
    // How many lines down from the first one, rounded to the nearest line.
    let lines_down = ((FIRST_LINE_Y - y) / LINE_STEP).round();
    if lines_down < 0.0 || lines_down >= LEVELS.len() as f32 {
        return None;
    }
    Some(lines_down as usize)
}

/// What the player asked for in the menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MenuCommand {
    Up,
    Down,
    /// Jump straight to the level with this number.
    Select(usize),
    Play,
}

/// The menu command a tap or swipe stands for, if any. Tapping a level
/// selects it; starting it takes the Play button, because on a phone the
/// lines are too close together to hit the right one every time.
fn menu_gesture(gesture: Gesture, button_shown: bool) -> Option<MenuCommand> {
    match gesture {
        Gesture::Swipe(Dir::Up) => Some(MenuCommand::Up),
        Gesture::Swipe(Dir::Down) => Some(MenuCommand::Down),
        Gesture::Swipe(_) => None,
        Gesture::Tap(at) if button_shown && PLAY_BUTTON.contains(at) => Some(MenuCommand::Play),
        Gesture::Tap(at) => line_at(at.y).map(MenuCommand::Select),
    }
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
    mut gestures: MessageReader<Gesture>,
    touch: Res<TouchMode>,
    mut selected: ResMut<Selected>,
    mut screen: ResMut<NextState<Screen>>,
) {
    // Everything asked for since the last frame: by keyboard, then by touch.
    let mut asked: Vec<MenuCommand> = keys_pressed(&mut keys).filter_map(menu_command).collect();
    asked.extend(
        gestures
            .read()
            .filter_map(|gesture| menu_gesture(*gesture, touch.0)),
    );

    for command in asked {
        match command {
            // The selection stops at both ends of the list.
            MenuCommand::Up if selected.0 > 0 => selected.0 -= 1,
            MenuCommand::Down if selected.0 + 1 < LEVELS.len() => selected.0 += 1,
            MenuCommand::Select(level) => selected.0 = level,
            MenuCommand::Play => screen.set(Screen::Playing),
            MenuCommand::Up | MenuCommand::Down => {}
        }
    }
}

/// Throws away the old picture and draws the list of levels.
pub fn draw_menu(
    mut commands: Commands,
    selected: Res<Selected>,
    progress: Res<Progress>,
    touch: Res<TouchMode>,
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

    let help = if touch.0 {
        view::spawn_button(&mut commands, PLAY_BUTTON);
        "Tap a kitchen, then Play"
    } else {
        "Up / Down: choose    Enter: play"
    };
    commands.spawn((Drawn, view::caption(help, 20.0, view::DIM_TEXT, HELP_Y)));
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
        let button_half = Button::SIZE.y / 2.0;
        // From the top down: title, level lines, Play button, help text,
        // with half a line of text above and below each line.
        assert!(TITLE_Y + 30.0 <= top);
        assert!(line_y(LEVELS.len() - 1) - LINE_STEP / 2.0 >= PLAY_BUTTON.centre.y + button_half);
        assert!(PLAY_BUTTON.centre.y - button_half >= HELP_Y + 10.0);
        assert!(HELP_Y - 15.0 >= -top);
    }

    #[test]
    fn a_menu_line_is_found_by_its_height() {
        assert_eq!(line_at(line_y(0)), Some(0));
        assert_eq!(line_at(line_y(3)), Some(3));
        assert_eq!(line_at(line_y(LEVELS.len() - 1)), Some(LEVELS.len() - 1));
    }

    #[test]
    fn a_tap_between_two_lines_picks_the_nearer_one() {
        assert_eq!(line_at(line_y(3) - LINE_STEP * 0.4), Some(3));
        assert_eq!(line_at(line_y(3) - LINE_STEP * 0.6), Some(4));
    }

    #[test]
    fn there_is_no_line_above_the_first_or_below_the_last() {
        assert_eq!(line_at(line_y(0) + LINE_STEP), None);
        assert_eq!(line_at(line_y(LEVELS.len() - 1) - LINE_STEP), None);
    }

    #[test]
    fn swiping_up_and_down_moves_the_selection() {
        assert_eq!(menu_gesture(Gesture::Swipe(Dir::Up), false), Some(MenuCommand::Up));
        assert_eq!(menu_gesture(Gesture::Swipe(Dir::Down), false), Some(MenuCommand::Down));
        assert_eq!(menu_gesture(Gesture::Swipe(Dir::Left), false), None);
    }

    #[test]
    fn tapping_a_level_selects_it() {
        let tap = Gesture::Tap(Vec2::new(40.0, line_y(5)));
        assert_eq!(menu_gesture(tap, true), Some(MenuCommand::Select(5)));
    }

    #[test]
    fn tapping_the_play_button_plays() {
        let tap = Gesture::Tap(PLAY_BUTTON.centre);
        assert_eq!(menu_gesture(tap, true), Some(MenuCommand::Play));
    }

    #[test]
    fn the_play_button_cannot_be_tapped_while_it_is_not_shown() {
        let tap = Gesture::Tap(PLAY_BUTTON.centre);
        assert_eq!(menu_gesture(tap, false), None);
    }

    #[test]
    fn other_keys_do_nothing_in_the_menu() {
        assert_eq!(menu_command(KeyCode::KeyZ), None);
    }
}
