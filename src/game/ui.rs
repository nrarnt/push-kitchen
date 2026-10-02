use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;

use super::input::keys_pressed;
use super::layout::{MenuLayout, ViewSize};
use super::levels::LEVELS;
use super::pointer::{Gesture, TouchMode};
use crate::puzzle::Dir;
use super::progress::Progress;
use super::view::{self, Drawn};
use super::{Screen, Selected};

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
fn menu_gesture(gesture: Gesture, button_shown: bool, layout: &MenuLayout) -> Option<MenuCommand> {
    match gesture {
        Gesture::Swipe(Dir::Up) => Some(MenuCommand::Up),
        Gesture::Swipe(Dir::Down) => Some(MenuCommand::Down),
        Gesture::Swipe(_) => None,
        Gesture::Tap(at) if button_shown && layout.play.contains(at) => Some(MenuCommand::Play),
        Gesture::Tap(at) => layout.line_at(at.y).map(MenuCommand::Select),
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
    view: Res<ViewSize>,
    mut selected: ResMut<Selected>,
    mut screen: ResMut<NextState<Screen>>,
) {
    // Everything asked for since the last frame: by keyboard, then by touch.
    let layout = MenuLayout::new(view.0, LEVELS.len());
    let mut asked: Vec<MenuCommand> = keys_pressed(&mut keys).filter_map(menu_command).collect();
    asked.extend(
        gestures
            .read()
            .filter_map(|gesture| menu_gesture(*gesture, touch.0, &layout)),
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
    view: Res<ViewSize>,
    drawn: Query<Entity, With<Drawn>>,
) {
    view::clear(&mut commands, &drawn);

    let layout = MenuLayout::new(view.0, LEVELS.len());
    commands.spawn((Drawn, view::caption("Push Kitchen", 36.0, view::LIGHT_TEXT, layout.title_y)));

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
        let line = view::caption(&text, layout.font_size, colour, layout.line_y(i));
        commands.spawn((Drawn, line));
    }

    let help = if touch.0 {
        view::spawn_button(&mut commands, layout.play);
        "Tap a kitchen, then Play"
    } else {
        "Up / Down: choose    Enter: play"
    };
    commands.spawn((Drawn, view::caption(help, 18.0, view::DIM_TEXT, layout.help_y)));
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

    /// The menu on a phone held upright.
    fn layout() -> MenuLayout {
        MenuLayout::new(Vec2::new(430.0, 900.0), LEVELS.len())
    }

    #[test]
    fn swiping_up_and_down_moves_the_selection() {
        let swiped = |dir| menu_gesture(Gesture::Swipe(dir), false, &layout());
        assert_eq!(swiped(Dir::Up), Some(MenuCommand::Up));
        assert_eq!(swiped(Dir::Down), Some(MenuCommand::Down));
        assert_eq!(swiped(Dir::Left), None);
    }

    #[test]
    fn tapping_a_level_selects_it() {
        let tap = Gesture::Tap(Vec2::new(40.0, layout().line_y(5)));
        assert_eq!(menu_gesture(tap, true, &layout()), Some(MenuCommand::Select(5)));
    }

    #[test]
    fn tapping_the_play_button_plays() {
        let tap = Gesture::Tap(layout().play.centre);
        assert_eq!(menu_gesture(tap, true, &layout()), Some(MenuCommand::Play));
    }

    #[test]
    fn the_play_button_cannot_be_tapped_while_it_is_not_shown() {
        let tap = Gesture::Tap(layout().play.centre);
        assert_eq!(menu_gesture(tap, false, &layout()), None);
    }

    #[test]
    fn other_keys_do_nothing_in_the_menu() {
        assert_eq!(menu_command(KeyCode::KeyZ), None);
    }
}
