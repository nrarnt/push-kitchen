//! Where things go on screen, worked out from the size of the window.
//!
//! The camera shows the window pixel for pixel, with (0, 0) in the middle and
//! `y` growing upwards. Everything here is plain arithmetic on that.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::pointer::Button;
use crate::puzzle::{Dir, Pos};

/// Space kept free along every edge of the window.
const MARGIN: f32 = 8.0;
/// Room above a kitchen for its name.
const TITLE_BAND: f32 = 56.0;
/// Room below a kitchen for the touch buttons or the key help.
const CONTROLS_BAND: f32 = 76.0;
/// Squares do not grow past this, so a small kitchen is not huge on a big screen.
const MAX_TILE: f32 = 96.0;

const BUTTON_HEIGHT: f32 = 56.0;
const BUTTON_MAX_WIDTH: f32 = 200.0;
const BUTTON_GAP: f32 = 10.0;

/// The size of the window or web page, in screen pixels.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct ViewSize(pub Vec2);

impl Default for ViewSize {
    fn default() -> Self {
        ViewSize(Vec2::new(800.0, 720.0))
    }
}

/// Keeps `ViewSize` equal to the size of the window, which changes when the
/// window is resized or a phone is turned.
pub fn track_window(window: Single<&Window, With<PrimaryWindow>>, mut view: ResMut<ViewSize>) {
    let size = Vec2::new(window.width(), window.height());
    if size.x > 0.0 && size.y > 0.0 && view.0 != size {
        view.0 = size;
    }
}

/// How one kitchen is laid out on the screen: as big as fits, with its name
/// above and the controls below.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoardLayout {
    /// Side of one square, in screen pixels.
    pub tile: f32,
    /// True if the kitchen is drawn with its rows and columns swapped, which
    /// makes a wide kitchen fill a tall screen (and the other way round).
    pub turned: bool,
    /// Columns and rows as drawn, after any turning.
    columns: i32,
    rows: i32,
    view: Vec2,
}

impl BoardLayout {
    /// The layout for a kitchen `width` squares wide and `height` high, in a
    /// window of size `view`.
    pub fn new(view: Vec2, width: i32, height: i32) -> BoardLayout {
        // The biggest square that lets `columns` fit across and `rows` fit
        // between the name and the controls.
        let tile_for = |columns: i32, rows: i32| {
            let across = (view.x - 2.0 * MARGIN) / columns as f32;
            let down = (view.y - TITLE_BAND - CONTROLS_BAND - 2.0 * MARGIN) / rows as f32;
            across.min(down).min(MAX_TILE)
        };

        // Turn the kitchen only if that makes its squares bigger.
        let turned = tile_for(height, width) > tile_for(width, height);
        let (columns, rows) = if turned { (height, width) } else { (width, height) };
        BoardLayout {
            tile: tile_for(columns, rows),
            turned,
            columns,
            rows,
            view,
        }
    }

    /// Height of the middle of the kitchen. The name, kitchen and controls
    /// are centred in the window as one block.
    fn centre_y(&self) -> f32 {
        (CONTROLS_BAND - TITLE_BAND) / 2.0
    }

    /// Where the middle of a square goes on screen.
    pub fn square_centre(&self, pos: Pos) -> Vec2 {
        let (column, row) = if self.turned { (pos.y, pos.x) } else { (pos.x, pos.y) };
        // Counted from the middle of the kitchen. On screen `y` grows
        // upwards, so the first row is the highest.
        let across = column as f32 - (self.columns - 1) as f32 / 2.0;
        let up = (self.rows - 1) as f32 / 2.0 - row as f32;
        Vec2::new(across * self.tile, self.centre_y() + up * self.tile)
    }

    /// A direction in the kitchen as it looks on screen, and also the other
    /// way round: swapping rows and columns twice changes nothing.
    pub fn turn(&self, dir: Dir) -> Dir {
        if self.turned { dir.transposed() } else { dir }
    }

    /// Height of the kitchen's name, just above it.
    pub fn title_y(&self) -> f32 {
        self.centre_y() + self.rows as f32 * self.tile / 2.0 + TITLE_BAND / 2.0
    }

    /// Height of the middle of the controls, just below the kitchen.
    pub fn controls_y(&self) -> f32 {
        self.centre_y() - self.rows as f32 * self.tile / 2.0 - CONTROLS_BAND / 2.0
    }

    /// Three buttons side by side under the kitchen, as wide as the window allows.
    pub fn buttons(&self, labels: [&'static str; 3]) -> [Button; 3] {
        // Three buttons and the four gaps around them share the width.
        let width = ((self.view.x - 4.0 * BUTTON_GAP) / 3.0).min(BUTTON_MAX_WIDTH);
        let step = width + BUTTON_GAP;
        let button = |label, x| Button {
            label,
            centre: Vec2::new(x, self.controls_y()),
            size: Vec2::new(width, BUTTON_HEIGHT),
        };
        let [left, middle, right] = labels;
        [button(left, -step), button(middle, 0.0), button(right, step)]
    }
}

/// How the menu is laid out: title, one line per level, the Play button and
/// a line of help, centred in the window as one block.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MenuLayout {
    pub title_y: f32,
    /// Distance from one level's line to the next.
    pub line_step: f32,
    pub font_size: f32,
    pub play: Button,
    pub help_y: f32,
    first_line_y: f32,
    lines: usize,
}

impl MenuLayout {
    const TITLE_BAND: f32 = 72.0;
    const PLAY_BAND: f32 = 80.0;
    const HELP_BAND: f32 = 32.0;
    /// Lines are this far apart when there is room: comfortable to tap.
    const LINE_STEP: f32 = 44.0;

    pub fn new(view: Vec2, lines: usize) -> MenuLayout {
        // The lines get whatever height the other three bands leave, and
        // move closer together if that is not enough.
        let bands = Self::TITLE_BAND + Self::PLAY_BAND + Self::HELP_BAND;
        let room = view.y - bands - 2.0 * MARGIN;
        let line_step = (room / lines as f32).min(Self::LINE_STEP);
        let list = lines as f32 * line_step;

        // Working down from the top of the block.
        let top = (bands + list) / 2.0;
        let list_top = top - Self::TITLE_BAND;
        let play_top = list_top - list;
        let help_top = play_top - Self::PLAY_BAND;
        MenuLayout {
            title_y: top - Self::TITLE_BAND / 2.0,
            line_step,
            font_size: (line_step * 0.55).min(22.0),
            play: Button {
                label: "Play",
                centre: Vec2::new(0.0, play_top - Self::PLAY_BAND / 2.0),
                size: Vec2::new(BUTTON_MAX_WIDTH.min(view.x - 2.0 * MARGIN), BUTTON_HEIGHT),
            },
            help_y: help_top - Self::HELP_BAND / 2.0,
            first_line_y: list_top - line_step / 2.0,
            lines,
        }
    }

    /// Height of the line of level number `i`.
    pub fn line_y(&self, i: usize) -> f32 {
        self.first_line_y - i as f32 * self.line_step
    }

    /// The number of the level whose line is at height `y`, if any.
    pub fn line_at(&self, y: f32) -> Option<usize> {
        // How many lines down from the first one, rounded to the nearest line.
        let lines_down = ((self.first_line_y - y) / self.line_step).round();
        if lines_down < 0.0 || lines_down >= self.lines as f32 {
            return None;
        }
        Some(lines_down as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::levels::LEVELS;

    const PHONE: Vec2 = Vec2::new(430.0, 900.0);
    const PHONE_SIDEWAYS: Vec2 = Vec2::new(900.0, 430.0);
    const SMALL_PHONE: Vec2 = Vec2::new(360.0, 640.0);
    const DESKTOP: Vec2 = Vec2::new(1280.0, 720.0);
    const ALL_SCREENS: [Vec2; 4] = [PHONE, PHONE_SIDEWAYS, SMALL_PHONE, DESKTOP];

    #[test]
    fn a_wide_kitchen_is_turned_on_a_tall_screen() {
        assert!(BoardLayout::new(PHONE, 9, 6).turned);
    }

    #[test]
    fn a_wide_kitchen_is_not_turned_on_a_wide_screen() {
        assert!(!BoardLayout::new(DESKTOP, 9, 6).turned);
    }

    #[test]
    fn a_tall_kitchen_is_turned_on_a_wide_screen() {
        assert!(BoardLayout::new(PHONE_SIDEWAYS, 5, 11).turned);
    }

    #[test]
    fn a_turned_kitchen_fills_the_width_of_a_phone() {
        // 9 wide and 6 high becomes 6 across: (430 - two margins) / 6.
        let layout = BoardLayout::new(PHONE, 9, 6);
        assert_eq!(layout.tile, (430.0 - 2.0 * MARGIN) / 6.0);
    }

    #[test]
    fn squares_do_not_grow_past_the_limit() {
        assert_eq!(BoardLayout::new(DESKTOP, 4, 3).tile, MAX_TILE);
    }

    #[test]
    fn the_middle_square_is_in_the_middle_of_the_kitchen() {
        let layout = BoardLayout::new(DESKTOP, 3, 3);
        assert_eq!(layout.square_centre(Pos::new(1, 1)), Vec2::new(0.0, layout.centre_y()));
    }

    #[test]
    fn columns_go_right_and_rows_go_down() {
        let layout = BoardLayout::new(DESKTOP, 3, 3);
        let middle = layout.square_centre(Pos::new(1, 1));
        assert_eq!(layout.square_centre(Pos::new(2, 1)), middle + Vec2::new(layout.tile, 0.0));
        // On screen `y` grows upwards, in the kitchen downwards.
        assert_eq!(layout.square_centre(Pos::new(1, 2)), middle - Vec2::new(0.0, layout.tile));
    }

    #[test]
    fn in_a_turned_kitchen_columns_go_down_and_rows_go_right() {
        let layout = BoardLayout::new(PHONE, 9, 6);
        let corner = layout.square_centre(Pos::new(0, 0));
        assert_eq!(layout.square_centre(Pos::new(1, 0)), corner - Vec2::new(0.0, layout.tile));
        assert_eq!(layout.square_centre(Pos::new(0, 1)), corner + Vec2::new(layout.tile, 0.0));
    }

    #[test]
    fn directions_are_swapped_only_in_a_turned_kitchen() {
        assert_eq!(BoardLayout::new(DESKTOP, 9, 6).turn(Dir::Right), Dir::Right);
        assert_eq!(BoardLayout::new(PHONE, 9, 6).turn(Dir::Right), Dir::Down);
    }

    #[test]
    fn every_kitchen_fits_every_screen_with_its_name_and_controls() {
        for view in ALL_SCREENS {
            for level in LEVELS {
                let board = level.board();
                let layout = BoardLayout::new(view, board.width(), board.height());
                let what = format!("{} on {view}", level.id);

                assert!(layout.tile > 0.0, "{what}");
                assert!(layout.columns as f32 * layout.tile <= view.x, "{what} is too wide");
                assert!(layout.title_y() + TITLE_BAND / 2.0 <= view.y / 2.0, "{what} is too tall");
                assert!(layout.controls_y() - CONTROLS_BAND / 2.0 >= -view.y / 2.0, "{what} is too tall");
            }
        }
    }

    #[test]
    fn the_buttons_sit_side_by_side_inside_the_window() {
        for view in ALL_SCREENS {
            let buttons = BoardLayout::new(view, 9, 6).buttons(["A", "B", "C"]);
            for pair in buttons.windows(2) {
                assert!(pair[0].centre.x + pair[0].size.x / 2.0 <= pair[1].centre.x - pair[1].size.x / 2.0);
            }
            for button in buttons {
                assert!(button.centre.x.abs() + button.size.x / 2.0 <= view.x / 2.0, "on {view}");
            }
        }
    }

    #[test]
    fn the_whole_menu_fits_every_screen() {
        for view in ALL_SCREENS {
            let menu = MenuLayout::new(view, LEVELS.len());
            let half_line = menu.line_step / 2.0;
            let last = LEVELS.len() - 1;
            let play_top = menu.play.centre.y + menu.play.size.y / 2.0;
            let play_bottom = menu.play.centre.y - menu.play.size.y / 2.0;

            // From the top down: title, level lines, Play button, help.
            assert!(menu.title_y + MenuLayout::TITLE_BAND / 2.0 <= view.y / 2.0, "on {view}");
            assert!(menu.line_y(0) + half_line <= menu.title_y, "on {view}");
            assert!(menu.line_y(last) - half_line >= play_top, "on {view}");
            assert!(play_bottom >= menu.help_y, "on {view}");
            assert!(menu.help_y - MenuLayout::HELP_BAND / 2.0 >= -view.y / 2.0, "on {view}");
        }
    }

    #[test]
    fn menu_lines_are_far_enough_apart_to_tap_on_a_phone() {
        assert_eq!(MenuLayout::new(PHONE, 12).line_step, MenuLayout::LINE_STEP);
    }

    #[test]
    fn a_menu_line_is_found_by_its_height() {
        let menu = MenuLayout::new(PHONE, 12);
        assert_eq!(menu.line_at(menu.line_y(0)), Some(0));
        assert_eq!(menu.line_at(menu.line_y(3)), Some(3));
        assert_eq!(menu.line_at(menu.line_y(11)), Some(11));
    }

    #[test]
    fn a_tap_between_two_lines_picks_the_nearer_one() {
        let menu = MenuLayout::new(PHONE, 12);
        assert_eq!(menu.line_at(menu.line_y(3) - menu.line_step * 0.4), Some(3));
        assert_eq!(menu.line_at(menu.line_y(3) - menu.line_step * 0.6), Some(4));
    }

    #[test]
    fn there_is_no_line_above_the_first_or_below_the_last() {
        let menu = MenuLayout::new(PHONE, 12);
        assert_eq!(menu.line_at(menu.line_y(0) + menu.line_step), None);
        assert_eq!(menu.line_at(menu.line_y(11) - menu.line_step), None);
    }
}
