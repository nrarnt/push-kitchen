use bevy::prelude::*;

use super::Selected;
use super::levels::LEVELS;
use super::session::Session;
use crate::puzzle::{Item, Pos, StationKind, Tile};

/// Side of one grid square, in pixels.
const TILE_SIZE: f32 = 64.0;
/// How much of its square an item fills.
const ITEM_SCALE: f32 = 0.7;

pub const BACKGROUND: Color = Color::srgb(0.10, 0.10, 0.13);
const CHEF: Color = Color::srgb(0.95, 0.95, 0.98);
const DARK_TEXT: Color = Color::srgb(0.12, 0.12, 0.14);
pub const LIGHT_TEXT: Color = Color::srgb(0.96, 0.96, 0.96);
pub const DIM_TEXT: Color = Color::srgb(0.60, 0.60, 0.66);

/// Marks everything a screen drew, so it can be cleared again.
#[derive(Component)]
pub struct Drawn;

/// Removes the picture of whichever screen was drawn last.
pub fn clear(commands: &mut Commands, drawn: &Query<Entity, With<Drawn>>) {
    for entity in drawn {
        commands.entity(entity).despawn();
    }
}

fn tile_colour(tile: Tile) -> Color {
    match tile {
        Tile::Floor => Color::srgb(0.87, 0.82, 0.72),
        Tile::Wall => Color::srgb(0.24, 0.26, 0.32),
        Tile::Station(StationKind::ChoppingBoard) => Color::srgb(0.58, 0.42, 0.28),
        Tile::Station(StationKind::Stove) => Color::srgb(0.30, 0.42, 0.60),
        Tile::Hatch(_) => Color::srgb(0.94, 0.94, 0.90),
    }
}

fn station_name(station: StationKind) -> &'static str {
    match station {
        StationKind::ChoppingBoard => "chop",
        StationKind::Stove => "stove",
    }
}

fn item_colour(item: Item) -> Color {
    match item {
        Item::Tomato => Color::srgb(0.88, 0.22, 0.18),
        Item::ChoppedTomato => Color::srgb(0.96, 0.50, 0.45),
        Item::TomatoSoup => Color::srgb(0.96, 0.58, 0.16),
        Item::Bread => Color::srgb(0.90, 0.76, 0.52),
        Item::Cheese => Color::srgb(0.98, 0.86, 0.26),
        Item::Sandwich => Color::srgb(0.80, 0.62, 0.36),
        Item::Toastie => Color::srgb(0.66, 0.44, 0.20),
    }
}

fn item_name(item: Item) -> &'static str {
    match item {
        Item::Tomato => "tomato",
        Item::ChoppedTomato => "chopped",
        Item::TomatoSoup => "soup",
        Item::Bread => "bread",
        Item::Cheese => "cheese",
        Item::Sandwich => "sandwich",
        Item::Toastie => "toastie",
    }
}

/// Where the middle of a grid square goes on screen, for a board of the given
/// size centred on the origin. Bevy's `y` grows upwards, the grid's downwards.
fn square_centre(pos: Pos, width: i32, height: i32) -> Vec2 {
    let x = pos.x as f32 - (width - 1) as f32 / 2.0;
    let y = (height - 1) as f32 / 2.0 - pos.y as f32;
    Vec2::new(x, y) * TILE_SIZE
}

/// A coloured square filling `scale` of a grid square. Higher layers are drawn on top.
fn square(colour: Color, scale: f32, centre: Vec2, layer: f32) -> (Sprite, Transform) {
    (
        Sprite::from_color(colour, Vec2::splat(TILE_SIZE * scale)),
        Transform::from_translation(centre.extend(layer)),
    )
}

/// Small text in the middle of a grid square.
fn label(
    text: &str,
    colour: Color,
    centre: Vec2,
    layer: f32,
) -> (Text2d, TextFont, TextColor, Transform) {
    (
        Text2d::new(text),
        TextFont::from_font_size(11.0),
        TextColor(colour),
        Transform::from_translation(centre.extend(layer)),
    )
}

/// A line of text centred at height `y`.
pub fn caption(
    text: &str,
    font_size: f32,
    colour: Color,
    y: f32,
) -> (Text2d, TextFont, TextColor, Transform) {
    (
        Text2d::new(text),
        TextFont::from_font_size(font_size),
        TextColor(colour),
        Transform::from_xyz(0.0, y, 3.0),
    )
}

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Throws away the old picture and draws the current board from scratch.
pub fn draw_board(
    mut commands: Commands,
    session: Res<Session>,
    selected: Res<Selected>,
    drawn: Query<Entity, With<Drawn>>,
) {
    clear(&mut commands, &drawn);

    let board = session.board();
    let (width, height) = (board.width(), board.height());

    for y in 0..height {
        for x in 0..width {
            let pos = Pos::new(x, y);
            let centre = square_centre(pos, width, height);

            let tile = board.tile(pos);
            commands.spawn((Drawn, square(tile_colour(tile), 0.96, centre, 0.0)));
            match tile {
                Tile::Station(station) => {
                    commands.spawn((Drawn, label(station_name(station), LIGHT_TEXT, centre, 0.3)));
                }
                Tile::Hatch(dish) => {
                    // A frame in the colour of the wanted dish, with a hole
                    // the dish fits into exactly.
                    commands.spawn((Drawn, square(item_colour(dish), 0.86, centre, 0.1)));
                    commands.spawn((Drawn, square(tile_colour(tile), ITEM_SCALE, centre, 0.2)));
                    commands.spawn((Drawn, label(item_name(dish), DARK_TEXT, centre, 0.3)));
                }
                Tile::Floor | Tile::Wall => {}
            }
        }
    }

    for (pos, item) in board.items() {
        let centre = square_centre(pos, width, height);
        commands.spawn((Drawn, square(item_colour(item), ITEM_SCALE, centre, 1.0)));
        commands.spawn((Drawn, label(item_name(item), DARK_TEXT, centre, 1.1)));
    }

    let chef = square_centre(board.chef(), width, height);
    commands.spawn((Drawn, square(CHEF, 0.5, chef, 2.0)));

    let (title, keys) = if board.is_solved() {
        ("Solved!", "Enter: next kitchen    Z: undo    Esc: menu")
    } else {
        (
            LEVELS[selected.0].name,
            "Arrows or WASD: move    Z: undo    R: restart    Esc: menu",
        )
    };
    let board_top = height as f32 * TILE_SIZE / 2.0;
    commands.spawn((Drawn, caption(title, 40.0, LIGHT_TEXT, board_top + 40.0)));
    commands.spawn((Drawn, caption(keys, 20.0, DIM_TEXT, -board_top - 30.0)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_middle_square_is_at_the_origin() {
        assert_eq!(square_centre(Pos::new(1, 1), 3, 3), Vec2::ZERO);
    }

    #[test]
    fn grid_x_grows_to_the_right() {
        assert_eq!(square_centre(Pos::new(2, 1), 3, 3), Vec2::new(TILE_SIZE, 0.0));
    }

    #[test]
    fn the_first_row_is_at_the_top() {
        assert_eq!(square_centre(Pos::new(1, 0), 3, 3), Vec2::new(0.0, TILE_SIZE));
    }

    #[test]
    fn an_even_board_is_centred_between_squares() {
        let half = TILE_SIZE / 2.0;
        assert_eq!(square_centre(Pos::new(0, 0), 2, 2), Vec2::new(-half, half));
    }
}
