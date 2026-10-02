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

    /// Buttons side by side under the kitchen, as wide as the window allows.
    pub fn buttons<const N: usize>(&self, labels: [&'static str; N]) -> [Button; N] {
        // The buttons and the gaps around them share the width.
        let count = N as f32;
        let width = ((self.view.x - (count + 1.0) * BUTTON_GAP) / count).min(BUTTON_MAX_WIDTH);
        let step = width + BUTTON_GAP;
        let mut place = 0.0;
        labels.map(|label| {
            // Counted from the middle of the row.
            let x = (place - (count - 1.0) / 2.0) * step;
            place += 1.0;
            Button {
                label,
                centre: Vec2::new(x, self.controls_y()),
                size: Vec2::new(width, BUTTON_HEIGHT),
            }
        })
    }
}

/// How the menu is laid out: title, one line per level, the Play button and
/// a line of help, centred in the window as one block. A list too long for
/// the window shows only some of its lines, and scrolls to keep the
/// selected one in view.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MenuLayout {
    pub title_y: f32,
    /// Distance from one level's line to the next.
    pub line_step: f32,
    pub font_size: f32,
    pub play: Button,
    pub help_y: f32,
    first_line_y: f32,
    /// The number of the level on the first line shown.
    first: usize,
    /// How many lines are shown.
    shown: usize,
    /// How many levels there are.
    lines: usize,
}

impl MenuLayout {
    const TITLE_BAND: f32 = 72.0;
    const PLAY_BAND: f32 = 80.0;
    const HELP_BAND: f32 = 32.0;
    /// Lines are this far apart when there is room: comfortable to tap.
    const LINE_STEP: f32 = 44.0;
    /// Lines are never closer together than this. If the whole list does
    /// not fit that way, only part of it is shown.
    const MIN_LINE_STEP: f32 = 34.0;

    /// The menu for a list of `lines` levels, of which number `selected` is
    /// the selected one.
    pub fn new(view: Vec2, lines: usize, selected: usize) -> MenuLayout {
        // The lines get whatever height the other three bands leave.
        let bands = Self::TITLE_BAND + Self::PLAY_BAND + Self::HELP_BAND;
        let room = view.y - bands - 2.0 * MARGIN;
        let fitting = ((room / Self::MIN_LINE_STEP) as usize).max(1);
        let shown = lines.min(fitting);
        let line_step = (room / shown as f32).min(Self::LINE_STEP);
        let list = shown as f32 * line_step;

        // The selected line sits in the middle of the lines shown, except
        // near the ends of the list, where there is nothing more to show.
        let first = selected.saturating_sub(shown / 2).min(lines - shown);

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
            first,
            shown,
            lines,
        }
    }

    /// The numbers of the levels whose lines are shown.
    pub fn shown_lines(&self) -> std::ops::Range<usize> {
        self.first..self.first + self.shown
    }

    /// Height of the line of level number `i`, which must be a shown one.
    pub fn line_y(&self, i: usize) -> f32 {
        self.first_line_y - (i - self.first) as f32 * self.line_step
    }

    /// The number of the level whose line is at height `y`, if any.
    pub fn line_at(&self, y: f32) -> Option<usize> {
        // How many lines down from the first one, rounded to the nearest line.
        let lines_down = ((self.first_line_y - y) / self.line_step).round();
        if lines_down < 0.0 || lines_down >= self.shown as f32 {
            return None;
        }
        Some(self.first + lines_down as usize)
    }

    /// Where to draw the mark that says there are more levels above the
    /// lines shown, if there are any. It sits just above the first line.
    pub fn more_above_y(&self) -> Option<f32> {
        let y = self.first_line_y + self.line_step / 2.0 + Self::MORE_MARK / 2.0;
        (self.first > 0).then_some(y)
    }

    /// The same for more levels below: just under the last line.
    pub fn more_below_y(&self) -> Option<f32> {
        let last_line_y = self.first_line_y - (self.shown - 1) as f32 * self.line_step;
        let y = last_line_y - self.line_step / 2.0 - Self::MORE_MARK / 2.0;
        (self.first + self.shown < self.lines).then_some(y)
    }

    /// Height of the "more levels" marks. They fit in the space the title
    /// and the Play button leave free at the edge of their bands.
    const MORE_MARK: f32 = 12.0;
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
    fn two_buttons_are_centred_under_the_kitchen() {
        let [left, right] = BoardLayout::new(PHONE, 9, 6).buttons(["A", "B"]);
        assert_eq!(left.centre.x, -right.centre.x);
        assert!(left.centre.x + left.size.x / 2.0 <= right.centre.x - right.size.x / 2.0);
    }

    #[test]
    fn the_whole_menu_fits_every_screen() {
        for view in ALL_SCREENS {
            let menu = MenuLayout::new(view, LEVELS.len(), 0);
            let half_line = menu.line_step / 2.0;
            let shown = menu.shown_lines();
            let play_top = menu.play.centre.y + menu.play.size.y / 2.0;
            let play_bottom = menu.play.centre.y - menu.play.size.y / 2.0;

            // From the top down: title, level lines, Play button, help.
            assert!(menu.title_y + MenuLayout::TITLE_BAND / 2.0 <= view.y / 2.0, "on {view}");
            assert!(menu.line_y(shown.start) + half_line <= menu.title_y, "on {view}");
            assert!(menu.line_y(shown.end - 1) - half_line >= play_top, "on {view}");
            assert!(play_bottom >= menu.help_y, "on {view}");
            assert!(menu.help_y - MenuLayout::HELP_BAND / 2.0 >= -view.y / 2.0, "on {view}");
        }
    }

    #[test]
    fn menu_lines_are_far_enough_apart_to_tap_on_a_phone() {
        assert_eq!(MenuLayout::new(PHONE, 12, 0).line_step, MenuLayout::LINE_STEP);
    }

    #[test]
    fn a_short_list_is_shown_whole() {
        assert_eq!(MenuLayout::new(PHONE, 12, 7).shown_lines(), 0..12);
    }

    #[test]
    fn a_long_list_shows_only_the_lines_that_fit_comfortably() {
        for view in ALL_SCREENS {
            let menu = MenuLayout::new(view, 40, 0);
            assert!(menu.shown_lines().len() < 40, "on {view}");
            assert!(menu.line_step >= MenuLayout::MIN_LINE_STEP, "on {view}");
        }
    }

    #[test]
    fn a_long_list_scrolls_to_keep_the_selected_line_in_view() {
        for view in ALL_SCREENS {
            for selected in 0..40 {
                let shown = MenuLayout::new(view, 40, selected).shown_lines();
                assert!(shown.contains(&selected), "line {selected} on {view}");
                assert!(shown.end <= 40, "line {selected} on {view}");
            }
        }
    }

    #[test]
    fn a_long_list_starts_at_the_top_and_ends_at_the_bottom() {
        assert_eq!(MenuLayout::new(SMALL_PHONE, 40, 0).shown_lines().start, 0);
        assert_eq!(MenuLayout::new(SMALL_PHONE, 40, 39).shown_lines().end, 40);
    }

    #[test]
    fn the_list_says_when_it_goes_on_above_or_below() {
        let top = MenuLayout::new(SMALL_PHONE, 40, 0);
        assert!(top.more_above_y().is_none() && top.more_below_y().is_some());
        let middle = MenuLayout::new(SMALL_PHONE, 40, 20);
        assert!(middle.more_above_y().is_some() && middle.more_below_y().is_some());
        let bottom = MenuLayout::new(SMALL_PHONE, 40, 39);
        assert!(bottom.more_above_y().is_some() && bottom.more_below_y().is_none());
        let short = MenuLayout::new(PHONE, 12, 5);
        assert!(short.more_above_y().is_none() && short.more_below_y().is_none());
    }

    #[test]
    fn a_menu_line_is_found_by_its_height() {
        let menu = MenuLayout::new(PHONE, 12, 0);
        assert_eq!(menu.line_at(menu.line_y(0)), Some(0));
        assert_eq!(menu.line_at(menu.line_y(3)), Some(3));
        assert_eq!(menu.line_at(menu.line_y(11)), Some(11));
    }

    #[test]
    fn a_line_of_a_scrolled_list_is_found_by_its_height() {
        let menu = MenuLayout::new(SMALL_PHONE, 40, 20);
        let shown = menu.shown_lines();
        assert!(shown.start > 0);
        assert_eq!(menu.line_at(menu.line_y(shown.start)), Some(shown.start));
        assert_eq!(menu.line_at(menu.line_y(20)), Some(20));
        assert_eq!(menu.line_at(menu.line_y(shown.end - 1)), Some(shown.end - 1));
    }

    #[test]
    fn a_tap_between_two_lines_picks_the_nearer_one() {
        let menu = MenuLayout::new(PHONE, 12, 0);
        assert_eq!(menu.line_at(menu.line_y(3) - menu.line_step * 0.4), Some(3));
        assert_eq!(menu.line_at(menu.line_y(3) - menu.line_step * 0.6), Some(4));
    }

    #[test]
    fn there_is_no_line_above_the_first_or_below_the_last() {
        let menu = MenuLayout::new(PHONE, 12, 0);
        assert_eq!(menu.line_at(menu.line_y(0) + menu.line_step), None);
        assert_eq!(menu.line_at(menu.line_y(11) - menu.line_step), None);
    }

    #[test]
    fn there_is_no_line_past_the_ends_of_a_scrolled_list() {
        let menu = MenuLayout::new(SMALL_PHONE, 40, 20);
        let shown = menu.shown_lines();
        assert_eq!(menu.line_at(menu.line_y(shown.start) + menu.line_step), None);
        assert_eq!(menu.line_at(menu.line_y(shown.end - 1) - menu.line_step), None);
    }
}
