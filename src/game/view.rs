use bevy::prelude::*;

use super::Selected;
use super::input::{self, Stage};
use super::layout::{BoardLayout, ViewSize};
use super::levels::LEVELS;
use super::pointer::{Button, TouchMode};
use super::session::Session;
use crate::puzzle::{Board, Dir, Item, Pos, StationKind, Tile};

/// How long a sprite takes to slide one square.
const SLIDE_SECONDS: f32 = 0.12;

pub const BACKGROUND: Color = Color::srgb(0.10, 0.10, 0.13);
pub const LIGHT_TEXT: Color = Color::srgb(0.96, 0.96, 0.96);
pub const DIM_TEXT: Color = Color::srgb(0.60, 0.60, 0.66);
const BUTTON: Color = Color::srgb(0.26, 0.29, 0.40);

const CHEF_SPRITE: &str = "sprites/chef.png";
/// The chef after stepping on something hot.
const BURNT_CHEF_SPRITE: &str = "sprites/chef_burnt.png";

/// The picture of a tile, as a file inside the `assets` folder.
fn tile_sprite(tile: Tile) -> &'static str {
    match tile {
        Tile::Floor => "sprites/floor.png",
        Tile::Wall => "sprites/wall.png",
        Tile::Station(StationKind::ChoppingBoard) => "sprites/chopping_board.png",
        Tile::Station(StationKind::Stove) => "sprites/stove.png",
        Tile::Hatch(_) => "sprites/hatch.png",
        Tile::Conveyor(_) => "sprites/conveyor.png",
        Tile::Ice => "sprites/ice.png",
        Tile::Bin => "sprites/bin.png",
    }
}

/// How far to turn the conveyor picture, which is drawn pointing up.
/// Bevy turns anticlockwise, in radians.
fn conveyor_turn(dir: Dir) -> f32 {
    use std::f32::consts::PI;
    match dir {
        Dir::Up => 0.0,
        Dir::Left => PI / 2.0,
        Dir::Down => PI,
        Dir::Right => -PI / 2.0,
    }
}

fn item_sprite(item: Item) -> &'static str {
    match item {
        Item::Tomato => "sprites/tomato.png",
        Item::ChoppedTomato => "sprites/chopped_tomato.png",
        Item::TomatoSoup => "sprites/tomato_soup.png",
        Item::Bread => "sprites/bread.png",
        Item::Cheese => "sprites/cheese.png",
        Item::Sandwich => "sprites/sandwich.png",
        Item::Toastie => "sprites/toastie.png",
    }
}

/// One tile of each kind that has its own picture.
const TILE_KINDS: [Tile; 8] = [
    Tile::Floor,
    Tile::Wall,
    Tile::Station(StationKind::ChoppingBoard),
    Tile::Station(StationKind::Stove),
    Tile::Hatch(Item::Tomato),
    Tile::Conveyor(Dir::Up),
    Tile::Ice,
    Tile::Bin,
];

/// Every picture file the game uses.
fn sprite_files() -> impl Iterator<Item = &'static str> {
    let items = Item::ALL.into_iter().map(item_sprite);
    let tiles = TILE_KINDS.into_iter().map(tile_sprite);
    items.chain(tiles).chain([CHEF_SPRITE, BURNT_CHEF_SPRITE])
}

/// Starts loading every picture, and returns the handles that keep them loaded.
pub fn preload(assets: &AssetServer) -> Vec<Handle<Image>> {
    sprite_files().map(|file| assets.load(file)).collect()
}

/// Marks everything a screen drew, so it can be cleared again.
#[derive(Component)]
pub struct Drawn;

/// Removes the picture of whichever screen was drawn last.
pub fn clear(commands: &mut Commands, drawn: &Query<Entity, With<Drawn>>) {
    for entity in drawn {
        commands.entity(entity).despawn();
    }
}

/// A square picture `size` pixels wide. Higher layers are drawn on top.
fn picture(image: Handle<Image>, size: f32, centre: Vec2, layer: f32) -> (Sprite, Transform) {
    (
        Sprite {
            image,
            custom_size: Some(Vec2::splat(size)),
            ..default()
        },
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

/// Draws a touch button: a coloured rectangle with its label on top.
pub fn spawn_button(commands: &mut Commands, button: Button) {
    commands.spawn((
        Drawn,
        Sprite::from_color(BUTTON, button.size),
        Transform::from_translation(button.centre.extend(3.0)),
    ));
    commands.spawn((
        Drawn,
        Text2d::new(button.label),
        TextFont::from_font_size(22.0),
        TextColor(LIGHT_TEXT),
        Transform::from_translation(button.centre.extend(3.1)),
    ));
}

/// Makes a sprite glide to its square from where it was a moment ago.
#[derive(Component)]
pub struct Slide {
    from: Vec2,
    to: Vec2,
    timer: Timer,
}

impl Slide {
    /// `tile` is the side of one square, in pixels.
    fn new(from: Vec2, to: Vec2, tile: f32) -> Self {
        // A longer way takes longer, so everything slides at the same speed.
        let squares = (from.distance(to) / tile).max(1.0);
        Slide {
            from,
            to,
            timer: Timer::from_seconds(squares * SLIDE_SECONDS, TimerMode::Once),
        }
    }

    /// Where the sprite is right now: quick at first, slowing as it arrives.
    fn position(&self) -> Vec2 {
        let left = 1.0 - self.timer.fraction();
        let eased = 1.0 - left * left;
        self.from.lerp(self.to, eased)
    }
}

/// Moves every sliding sprite a little further along, each frame.
pub fn slide(time: Res<Time>, mut sliding: Query<(&mut Slide, &mut Transform)>) {
    for (mut slide, mut transform) in &mut sliding {
        slide.timer.tick(time.delta());
        let position = slide.position();
        transform.translation.x = position.x;
        transform.translation.y = position.y;
    }
}

/// The square the item now on `pos` came from, if the change from `before`
/// to `after` moved it there.
///
/// The board does not know which item is which, so this is a guess: the
/// nearest other square whose item is gone or different now.
fn item_origin(before: &Board, after: &Board, pos: Pos) -> Option<Pos> {
    if before.item(pos) == after.item(pos) {
        // Same item as before: it never moved.
        return None;
    }
    before
        .items()
        .filter(|&(from, item)| from != pos && after.item(from) != Some(item))
        .map(|(from, _)| from)
        .min_by_key(|from| (from.x - pos.x).abs() + (from.y - pos.y).abs())
}

/// Draws something that can move, filling `scale` of a square `tile` pixels
/// wide. It belongs on `to`; if it was on `from` a moment ago, it starts
/// there and slides over.
fn spawn_piece(
    commands: &mut Commands,
    image: Handle<Image>,
    tile: f32,
    scale: f32,
    layer: f32,
    from: Option<Vec2>,
    to: Vec2,
) {
    let size = tile * scale;
    match from {
        Some(from) => commands.spawn((
            Drawn,
            picture(image, size, from, layer),
            Slide::new(from, to, tile),
        )),
        None => commands.spawn((Drawn, picture(image, size, to, layer))),
    };
}

/// The camera shows the window pixel for pixel, with (0, 0) in the middle.
pub fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Throws away the old picture and draws the current board from scratch.
/// Whatever the latest change moved starts on its old square and slides over.
pub fn draw_board(
    mut commands: Commands,
    assets: Res<AssetServer>,
    session: Res<Session>,
    selected: Res<Selected>,
    touch: Res<TouchMode>,
    view: Res<ViewSize>,
    drawn: Query<Entity, With<Drawn>>,
) {
    clear(&mut commands, &drawn);

    let board = session.board();
    let layout = BoardLayout::new(view.0, board.width(), board.height());
    let tile_size = layout.tile;
    let centre = |pos: Pos| layout.square_centre(pos);

    for y in 0..board.height() {
        for x in 0..board.width() {
            let pos = Pos::new(x, y);
            let tile = board.tile(pos);
            let (sprite, mut transform) =
                picture(assets.load(tile_sprite(tile)), tile_size, centre(pos), 0.0);
            if let Tile::Conveyor(dir) = tile {
                // The arrows point the way items go as seen on screen.
                transform.rotate_z(conveyor_turn(layout.turn(dir)));
            }
            commands.spawn((Drawn, sprite, transform));

            if let Tile::Hatch(dish) = tile {
                // A faint picture of the dish this hatch wants.
                let image = assets.load(item_sprite(dish));
                let (mut sprite, transform) = picture(image, tile_size * 0.8, centre(pos), 0.5);
                sprite.color = Color::WHITE.with_alpha(0.35);
                commands.spawn((Drawn, sprite, transform));
            }
        }
    }

    // Things slide only when the board itself has just changed, not when the
    // picture is redrawn for another reason, such as a resized window.
    let previous = if session.is_changed() { session.previous() } else { None };

    for (pos, item) in board.items() {
        let image = assets.load(item_sprite(item));
        let from = previous
            .and_then(|before| item_origin(before, board, pos))
            .map(centre);
        spawn_piece(&mut commands, image, tile_size, 0.8, 1.0, from, centre(pos));
    }

    let stage = Stage::of(board);

    let chef = if stage == Stage::Burnt { BURNT_CHEF_SPRITE } else { CHEF_SPRITE };
    let image = assets.load(chef);
    let from = previous.map(|before| centre(before.chef()));
    spawn_piece(&mut commands, image, tile_size, 0.9, 2.0, from, centre(board.chef()));

    let title = match stage {
        Stage::Cooking => LEVELS[selected.0].name,
        Stage::Solved => "Solved!",
        Stage::Burnt => "Burnt!",
    };
    commands.spawn((Drawn, caption(title, 30.0, LIGHT_TEXT, layout.title_y())));

    // Under the board: buttons for fingers, or a reminder of the keys.
    if touch.0 {
        for button in input::buttons(stage, &layout) {
            spawn_button(&mut commands, button);
        }
    } else {
        let keys = match stage {
            Stage::Cooking => "Arrows or WASD: move    Z: undo    R: restart    Esc: menu",
            Stage::Solved => "Enter: next kitchen    Z: undo    Esc: menu",
            Stage::Burnt => "R: start again    Esc: menu",
        };
        // Smaller in a narrow window, so the whole line stays in view.
        let font_size = (view.0.x / 36.0).min(18.0);
        commands.spawn((Drawn, caption(keys, font_size, DIM_TEXT, layout.controls_y())));
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::time::Duration;

    use super::*;
    use crate::puzzle::parse;

    fn board(text: &str) -> Board {
        parse(text).expect("test level should parse")
    }

    /// The side of a square in these tests.
    const TILE: f32 = 64.0;

    #[test]
    fn a_slide_starts_where_the_sprite_was() {
        let slide = Slide::new(Vec2::ZERO, Vec2::new(TILE, 0.0), TILE);
        assert_eq!(slide.position(), Vec2::ZERO);
    }

    #[test]
    fn a_slide_ends_on_the_sprites_square() {
        let mut slide = Slide::new(Vec2::ZERO, Vec2::new(TILE, 0.0), TILE);
        slide.timer.tick(Duration::from_secs_f32(SLIDE_SECONDS));
        assert_eq!(slide.position(), Vec2::new(64.0, 0.0));
    }

    #[test]
    fn a_slide_is_past_halfway_at_half_time() {
        let mut slide = Slide::new(Vec2::ZERO, Vec2::new(TILE, 0.0), TILE);
        slide.timer.tick(Duration::from_secs_f32(SLIDE_SECONDS / 2.0));
        assert!(slide.position().x > 32.0);
        assert!(slide.position().x < 64.0);
    }

    #[test]
    fn a_slide_over_several_squares_takes_that_much_longer() {
        let slide = Slide::new(Vec2::ZERO, Vec2::new(3.0 * TILE, 0.0), TILE);
        assert_eq!(
            slide.timer.duration(),
            Duration::from_secs_f32(3.0 * SLIDE_SECONDS)
        );
    }

    #[test]
    fn an_item_that_slid_far_comes_from_where_it_started() {
        let before = board("#@t**..#");
        let after = before.step(Dir::Right).unwrap();
        assert_eq!(item_origin(&before, &after, Pos::new(5, 0)), Some(Pos::new(2, 0)));
    }

    #[test]
    fn a_pushed_item_comes_from_the_square_the_chef_took() {
        let before = board("#@t.#");
        let after = before.step(Dir::Right).unwrap();
        assert_eq!(item_origin(&before, &after, Pos::new(3, 0)), Some(Pos::new(2, 0)));
    }

    #[test]
    fn an_item_that_was_not_pushed_has_no_origin() {
        let before = board("#b@t.#");
        let after = before.step(Dir::Right).unwrap();
        assert_eq!(item_origin(&before, &after, Pos::new(1, 0)), None);
    }

    #[test]
    fn a_combined_item_comes_from_the_square_of_the_pushed_one() {
        let before = board("#@bc#");
        let after = before.step(Dir::Right).unwrap();
        assert_eq!(item_origin(&before, &after, Pos::new(3, 0)), Some(Pos::new(2, 0)));
    }

    #[test]
    fn undoing_a_push_slides_the_item_back() {
        let after = board("#@t.#");
        let before = after.step(Dir::Right).unwrap();
        assert_eq!(item_origin(&before, &after, Pos::new(2, 0)), Some(Pos::new(3, 0)));
    }

    #[test]
    fn undoing_a_combine_slides_only_the_pushed_item_back() {
        let after = board("#@bc#");
        let before = after.step(Dir::Right).unwrap();
        // The bread goes back; the cheese reappears where the sandwich was.
        assert_eq!(item_origin(&before, &after, Pos::new(2, 0)), Some(Pos::new(3, 0)));
        assert_eq!(item_origin(&before, &after, Pos::new(3, 0)), None);
    }

    #[test]
    fn every_picture_has_its_file() {
        for file in sprite_files() {
            let path = Path::new("assets").join(file);
            assert!(path.is_file(), "{} is missing", path.display());
        }
    }
}
