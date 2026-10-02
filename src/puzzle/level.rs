use std::collections::HashMap;

use super::board::Board;
use super::types::{Dir, Item, Pos, StationKind, Tile};

/// Why a level file could not be turned into a `Board`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelError {
    NoChef,
    ExtraChef,
    UnknownSymbol(char),
}

/// The item a lowercase letter stands for.
fn item_for(symbol: char) -> Option<Item> {
    match symbol {
        't' => Some(Item::Tomato),
        'd' => Some(Item::ChoppedTomato),
        's' => Some(Item::TomatoSoup),
        'b' => Some(Item::Bread),
        'c' => Some(Item::Cheese),
        'w' => Some(Item::Sandwich),
        'g' => Some(Item::Toastie),
        _ => None,
    }
}

/// Builds a `Board` from a text drawing of a kitchen.
///
/// `#` wall, `.` floor, `@` chef, `/` chopping board, `~` stove,
/// `^` `v` `<` `>` conveyors, `*` ice, `x` bin.
///
/// A lowercase letter is an item on the floor: `t` tomato, `d` chopped
/// (diced) tomato, `s` tomato soup, `b` bread, `c` cheese, `w` sandwich,
/// `g` toastie (grilled). Its capital is a hatch that wants that item.
pub fn parse(text: &str) -> Result<Board, LevelError> {
    let lines: Vec<&str> = text.lines().collect();
    let width = lines.iter().map(|line| line.chars().count()).max().unwrap_or(0);
    let height = lines.len();

    // Rows shorter than the widest one are padded with wall.
    let mut tiles = vec![Tile::Wall; width * height];
    let mut items = HashMap::new();
    let mut chef = None;

    for (y, line) in lines.iter().enumerate() {
        for (x, symbol) in line.chars().enumerate() {
            let pos = Pos::new(x as i32, y as i32);
            let tile = match symbol {
                '#' => Tile::Wall,
                '.' => Tile::Floor,
                '/' => Tile::Station(StationKind::ChoppingBoard),
                '~' => Tile::Station(StationKind::Stove),
                '^' => Tile::Conveyor(Dir::Up),
                'v' => Tile::Conveyor(Dir::Down),
                '<' => Tile::Conveyor(Dir::Left),
                '>' => Tile::Conveyor(Dir::Right),
                '*' => Tile::Ice,
                'x' => Tile::Bin,
                '@' => {
                    if chef.is_some() {
                        return Err(LevelError::ExtraChef);
                    }
                    chef = Some(pos);
                    Tile::Floor
                }
                // A letter: first try it as an item, then as the capital of one.
                other => match (item_for(other), item_for(other.to_ascii_lowercase())) {
                    (Some(item), _) => {
                        items.insert(pos, item);
                        Tile::Floor
                    }
                    (None, Some(dish)) => Tile::Hatch(dish),
                    (None, None) => return Err(LevelError::UnknownSymbol(other)),
                },
            };
            tiles[y * width + x] = tile;
        }
    }

    let chef = chef.ok_or(LevelError::NoChef)?;
    Ok(Board::new(width as i32, height as i32, tiles, items, chef))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_finds_the_chef() {
        let board = parse("###\n#@#\n###").unwrap();
        assert_eq!(board.chef(), Pos::new(1, 1));
    }

    #[test]
    fn parse_reads_walls_and_floor() {
        let board = parse("#@.").unwrap();
        assert_eq!(board.tile(Pos::new(0, 0)), Tile::Wall);
        assert_eq!(board.tile(Pos::new(1, 0)), Tile::Floor);
        assert_eq!(board.tile(Pos::new(2, 0)), Tile::Floor);
    }

    #[test]
    fn parse_reads_stations() {
        let board = parse("@/~").unwrap();
        assert_eq!(
            board.tile(Pos::new(1, 0)),
            Tile::Station(StationKind::ChoppingBoard)
        );
        assert_eq!(board.tile(Pos::new(2, 0)), Tile::Station(StationKind::Stove));
    }

    #[test]
    fn parse_reads_conveyors_with_their_direction() {
        let board = parse("@^v<>").unwrap();
        assert_eq!(board.tile(Pos::new(1, 0)), Tile::Conveyor(Dir::Up));
        assert_eq!(board.tile(Pos::new(2, 0)), Tile::Conveyor(Dir::Down));
        assert_eq!(board.tile(Pos::new(3, 0)), Tile::Conveyor(Dir::Left));
        assert_eq!(board.tile(Pos::new(4, 0)), Tile::Conveyor(Dir::Right));
    }

    #[test]
    fn parse_reads_ice_and_the_bin() {
        let board = parse("@*x").unwrap();
        assert_eq!(board.tile(Pos::new(1, 0)), Tile::Ice);
        assert_eq!(board.tile(Pos::new(2, 0)), Tile::Bin);
    }

    #[test]
    fn parse_places_a_lowercase_letter_as_an_item_on_floor() {
        let board = parse("@tdsbcwg").unwrap();
        let expected = [
            Item::Tomato,
            Item::ChoppedTomato,
            Item::TomatoSoup,
            Item::Bread,
            Item::Cheese,
            Item::Sandwich,
            Item::Toastie,
        ];
        for (i, item) in expected.into_iter().enumerate() {
            let pos = Pos::new(i as i32 + 1, 0);
            assert_eq!(board.item(pos), Some(item));
            assert_eq!(board.tile(pos), Tile::Floor);
        }
    }

    #[test]
    fn parse_reads_a_capital_letter_as_a_hatch_wanting_that_item() {
        let board = parse("@SG").unwrap();
        assert_eq!(board.tile(Pos::new(1, 0)), Tile::Hatch(Item::TomatoSoup));
        assert_eq!(board.tile(Pos::new(2, 0)), Tile::Hatch(Item::Toastie));
        assert_eq!(board.item(Pos::new(1, 0)), None);
    }

    #[test]
    fn squares_outside_the_drawing_are_walls() {
        let board = parse("@.\n.").unwrap();
        assert_eq!(board.tile(Pos::new(1, 1)), Tile::Wall);
        assert_eq!(board.tile(Pos::new(-1, 0)), Tile::Wall);
        assert_eq!(board.tile(Pos::new(0, 5)), Tile::Wall);
    }

    #[test]
    fn level_without_a_chef_is_rejected() {
        assert_eq!(parse("#.#"), Err(LevelError::NoChef));
    }

    #[test]
    fn level_with_two_chefs_is_rejected() {
        assert_eq!(parse("@.@"), Err(LevelError::ExtraChef));
    }

    #[test]
    fn unknown_symbol_is_rejected() {
        assert_eq!(parse("@?"), Err(LevelError::UnknownSymbol('?')));
    }

    #[test]
    fn a_capital_letter_that_is_no_item_is_rejected() {
        assert_eq!(parse("@X"), Err(LevelError::UnknownSymbol('X')));
    }
}
