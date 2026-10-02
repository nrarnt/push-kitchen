use bevy::prelude::*;

use super::session::Session;
use crate::puzzle::{Item, Pos, Tile};

/// Side of one grid square, in pixels.
const TILE_SIZE: f32 = 64.0;

pub const BACKGROUND: Color = Color::srgb(0.10, 0.10, 0.13);
const CHEF: Color = Color::srgb(0.95, 0.95, 0.98);

/// Marks everything `draw_board` put on screen, so it can be cleared again.
#[derive(Component)]
pub struct Drawn;

fn tile_colour(tile: Tile) -> Color {
    match tile {
        Tile::Floor => Color::srgb(0.87, 0.82, 0.72),
        Tile::Wall => Color::srgb(0.24, 0.26, 0.32),
        Tile::Hatch => Color::srgb(0.45, 0.75, 0.50),
    }
}

fn item_colour(item: Item) -> Color {
    match item {
        Item::Crate => Color::srgb(0.62, 0.42, 0.24),
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

/// A line of text centred at height `y`.
fn caption(text: &str, font_size: f32, y: f32) -> (Text2d, TextFont, Transform) {
    (
        Text2d::new(text),
        TextFont::from_font_size(font_size),
        Transform::from_xyz(0.0, y, 3.0),
    )
}

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Throws away the old picture and draws the current board from scratch.
pub fn draw_board(mut commands: Commands, session: Res<Session>, drawn: Query<Entity, With<Drawn>>) {
    for entity in &drawn {
        commands.entity(entity).despawn();
    }

    let board = session.board();
    let (width, height) = (board.width(), board.height());

    for y in 0..height {
        for x in 0..width {
            let pos = Pos::new(x, y);
            let centre = square_centre(pos, width, height);
            commands.spawn((Drawn, square(tile_colour(board.tile(pos)), 0.96, centre, 0.0)));
            if let Some(item) = board.item(pos) {
                commands.spawn((Drawn, square(item_colour(item), 0.7, centre, 1.0)));
            }
        }
    }

    let chef = square_centre(board.chef(), width, height);
    commands.spawn((Drawn, square(CHEF, 0.5, chef, 2.0)));

    let board_top = height as f32 * TILE_SIZE / 2.0;
    let keys = "Arrows or WASD: move    Z: undo    R: restart";
    commands.spawn((Drawn, caption(keys, 20.0, -board_top - 30.0)));
    if board.is_solved() {
        commands.spawn((Drawn, caption("Solved!", 48.0, board_top + 40.0)));
    }
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
