use bevy::camera::ScalingMode;
use bevy::prelude::*;

use super::Selected;
use super::levels::LEVELS;
use super::session::Session;
use crate::puzzle::{Board, Dir, Item, Pos, StationKind, Tile};

/// Side of one grid square, in pixels.
const TILE_SIZE: f32 = 64.0;
/// The part of the scene that is always in view, whatever the shape of the
/// window or web page. A bigger window shows it larger.
pub const VIEW_WIDTH: f32 = 800.0;
pub const VIEW_HEIGHT: f32 = 720.0;
/// How long a sprite takes to slide one square.
const SLIDE_SECONDS: f32 = 0.12;

pub const BACKGROUND: Color = Color::srgb(0.10, 0.10, 0.13);
pub const LIGHT_TEXT: Color = Color::srgb(0.96, 0.96, 0.96);
pub const DIM_TEXT: Color = Color::srgb(0.60, 0.60, 0.66);

const CHEF_SPRITE: &str = "sprites/chef.png";

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
    items.chain(tiles).chain([CHEF_SPRITE])
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

/// Where the middle of a grid square goes on screen, for a board of the given
/// size centred on the origin. Bevy's `y` grows upwards, the grid's downwards.
fn square_centre(pos: Pos, width: i32, height: i32) -> Vec2 {
    let x = pos.x as f32 - (width - 1) as f32 / 2.0;
    let y = (height - 1) as f32 / 2.0 - pos.y as f32;
    Vec2::new(x, y) * TILE_SIZE
}

/// A picture filling `scale` of a grid square. Higher layers are drawn on top.
fn picture(image: Handle<Image>, scale: f32, centre: Vec2, layer: f32) -> (Sprite, Transform) {
    (
        Sprite {
            image,
            custom_size: Some(Vec2::splat(TILE_SIZE * scale)),
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

/// Makes a sprite glide to its square from where it was a moment ago.
#[derive(Component)]
pub struct Slide {
    from: Vec2,
    to: Vec2,
    timer: Timer,
}

impl Slide {
    fn new(from: Vec2, to: Vec2) -> Self {
        // A longer way takes longer, so everything slides at the same speed.
        let squares = (from.distance(to) / TILE_SIZE).max(1.0);
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

/// Draws something that can move. It belongs on `to`; if it was on `from` a
/// moment ago, it starts there and slides over.
fn spawn_piece(
    commands: &mut Commands,
    image: Handle<Image>,
    scale: f32,
    layer: f32,
    from: Option<Vec2>,
    to: Vec2,
) {
    match from {
        Some(from) => commands.spawn((
            Drawn,
            picture(image, scale, from, layer),
            Slide::new(from, to),
        )),
        None => commands.spawn((Drawn, picture(image, scale, to, layer))),
    };
}

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: VIEW_WIDTH,
                min_height: VIEW_HEIGHT,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}

/// Throws away the old picture and draws the current board from scratch.
/// Whatever the latest change moved starts on its old square and slides over.
pub fn draw_board(
    mut commands: Commands,
    assets: Res<AssetServer>,
    session: Res<Session>,
    selected: Res<Selected>,
    drawn: Query<Entity, With<Drawn>>,
) {
    clear(&mut commands, &drawn);

    let board = session.board();
    let (width, height) = (board.width(), board.height());
    let centre = |pos: Pos| square_centre(pos, width, height);

    for y in 0..height {
        for x in 0..width {
            let pos = Pos::new(x, y);
            let tile = board.tile(pos);
            let (sprite, mut transform) =
                picture(assets.load(tile_sprite(tile)), 1.0, centre(pos), 0.0);
            if let Tile::Conveyor(dir) = tile {
                transform.rotate_z(conveyor_turn(dir));
            }
            commands.spawn((Drawn, sprite, transform));

            if let Tile::Hatch(dish) = tile {
                // A faint picture of the dish this hatch wants.
                let (mut sprite, transform) =
                    picture(assets.load(item_sprite(dish)), 0.8, centre(pos), 0.5);
                sprite.color = Color::WHITE.with_alpha(0.35);
                commands.spawn((Drawn, sprite, transform));
            }
        }
    }

    let previous = session.previous();

    for (pos, item) in board.items() {
        let image = assets.load(item_sprite(item));
        let from = previous
            .and_then(|before| item_origin(before, board, pos))
            .map(centre);
        spawn_piece(&mut commands, image, 0.8, 1.0, from, centre(pos));
    }

    let image = assets.load(CHEF_SPRITE);
    let from = previous.map(|before| centre(before.chef()));
    spawn_piece(&mut commands, image, 0.9, 2.0, from, centre(board.chef()));

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
    use std::path::Path;
    use std::time::Duration;

    use super::*;
    use crate::puzzle::parse;

    fn board(text: &str) -> Board {
        parse(text).expect("test level should parse")
    }

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

    #[test]
    fn a_slide_starts_where_the_sprite_was() {
        let slide = Slide::new(Vec2::ZERO, Vec2::new(64.0, 0.0));
        assert_eq!(slide.position(), Vec2::ZERO);
    }

    #[test]
    fn a_slide_ends_on_the_sprites_square() {
        let mut slide = Slide::new(Vec2::ZERO, Vec2::new(64.0, 0.0));
        slide.timer.tick(Duration::from_secs_f32(SLIDE_SECONDS));
        assert_eq!(slide.position(), Vec2::new(64.0, 0.0));
    }

    #[test]
    fn a_slide_is_past_halfway_at_half_time() {
        let mut slide = Slide::new(Vec2::ZERO, Vec2::new(64.0, 0.0));
        slide.timer.tick(Duration::from_secs_f32(SLIDE_SECONDS / 2.0));
        assert!(slide.position().x > 32.0);
        assert!(slide.position().x < 64.0);
    }

    #[test]
    fn a_slide_over_several_squares_takes_that_much_longer() {
        let slide = Slide::new(Vec2::ZERO, Vec2::new(3.0 * TILE_SIZE, 0.0));
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

    #[test]
    fn every_level_fits_in_the_view() {
        // Room for the title above the board and the key help below it.
        let captions = 2.0 * 64.0;
        for level in LEVELS {
            let board = level.board();
            let width = board.width() as f32 * TILE_SIZE;
            let height = board.height() as f32 * TILE_SIZE + captions;
            assert!(width <= VIEW_WIDTH, "{} is too wide", level.id);
            assert!(height <= VIEW_HEIGHT, "{} is too tall", level.id);
        }
    }
}
