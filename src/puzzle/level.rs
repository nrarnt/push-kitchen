use std::collections::HashMap;

use super::board::Board;
use super::types::{Item, Pos, Tile};

/// Why a level file could not be turned into a `Board`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelError {
    NoChef,
    ExtraChef,
    UnknownSymbol(char),
}

/// Builds a `Board` from a text drawing of a kitchen.
///
/// `#` wall, `.` floor, `@` chef, `c` crate, `H` hatch, `C` crate on a hatch.
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
                'H' => Tile::Hatch,
                '@' => {
                    if chef.is_some() {
                        return Err(LevelError::ExtraChef);
                    }
                    chef = Some(pos);
                    Tile::Floor
                }
                'c' => {
                    items.insert(pos, Item::Crate);
                    Tile::Floor
                }
                'C' => {
                    items.insert(pos, Item::Crate);
                    Tile::Hatch
                }
                other => return Err(LevelError::UnknownSymbol(other)),
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
    fn parse_reads_walls_floor_and_hatches() {
        let board = parse("#@.H").unwrap();
        assert_eq!(board.tile(Pos::new(0, 0)), Tile::Wall);
        assert_eq!(board.tile(Pos::new(1, 0)), Tile::Floor);
        assert_eq!(board.tile(Pos::new(2, 0)), Tile::Floor);
        assert_eq!(board.tile(Pos::new(3, 0)), Tile::Hatch);
    }

    #[test]
    fn parse_places_crates_on_floor_and_on_hatches() {
        let board = parse("@cC").unwrap();
        assert_eq!(board.item(Pos::new(1, 0)), Some(Item::Crate));
        assert_eq!(board.tile(Pos::new(1, 0)), Tile::Floor);
        assert_eq!(board.item(Pos::new(2, 0)), Some(Item::Crate));
        assert_eq!(board.tile(Pos::new(2, 0)), Tile::Hatch);
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
}
