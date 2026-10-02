use std::collections::HashMap;

use super::types::{Dir, Item, Pos, Tile};

/// The whole state of one kitchen at one moment.
#[derive(Debug, Clone, PartialEq)]
pub struct Board {
    width: i32,
    height: i32,
    /// Row by row, `width * height` long.
    tiles: Vec<Tile>,
    items: HashMap<Pos, Item>,
    chef: Pos,
}

impl Board {
    pub fn new(
        width: i32,
        height: i32,
        tiles: Vec<Tile>,
        items: HashMap<Pos, Item>,
        chef: Pos,
    ) -> Self {
        Board {
            width,
            height,
            tiles,
            items,
            chef,
        }
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn chef(&self) -> Pos {
        self.chef
    }

    /// The tile at `pos`. Anything outside the kitchen counts as wall.
    pub fn tile(&self, pos: Pos) -> Tile {
        let inside = pos.x >= 0 && pos.x < self.width && pos.y >= 0 && pos.y < self.height;
        if !inside {
            return Tile::Wall;
        }
        self.tiles[(pos.y * self.width + pos.x) as usize]
    }

    pub fn item(&self, pos: Pos) -> Option<Item> {
        self.items.get(&pos).copied()
    }

    /// The board after the chef tries to move one square in `dir`,
    /// or `None` if the move is not allowed.
    pub fn step(&self, dir: Dir) -> Option<Board> {
        let dest = self.chef.step(dir);
        if self.tile(dest) == Tile::Wall {
            return None;
        }

        let mut next = self.clone();
        if let Some(item) = next.items.remove(&dest) {
            // The chef walks into an item and shoves it one square further,
            // unless a wall or another item is in the way.
            let beyond = dest.step(dir);
            if self.tile(beyond) == Tile::Wall || self.item(beyond).is_some() {
                return None;
            }
            next.items.insert(beyond, item);
        }
        next.chef = dest;
        Some(next)
    }

    pub fn is_solved(&self) -> bool {
        for y in 0..self.height {
            for x in 0..self.width {
                let pos = Pos::new(x, y);
                if self.tile(pos) == Tile::Hatch && self.item(pos).is_none() {
                    return false;
                }
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::level::parse;

    fn board(text: &str) -> Board {
        parse(text).expect("test level should parse")
    }

    #[test]
    fn board_knows_its_size() {
        let board = board("#@.#\n####");
        assert_eq!(board.width(), 4);
        assert_eq!(board.height(), 2);
    }

    #[test]
    fn chef_walks_onto_empty_floor() {
        let before = board("#@.#");
        assert_eq!(before.step(Dir::Right), Some(board("#.@#")));
    }

    #[test]
    fn chef_is_blocked_by_wall() {
        let before = board("#@.#");
        assert_eq!(before.step(Dir::Left), None);
    }

    #[test]
    fn chef_pushes_crate_onto_empty_floor() {
        let before = board("#@c.#");
        assert_eq!(before.step(Dir::Right), Some(board("#.@c#")));
    }

    #[test]
    fn push_is_blocked_by_wall_behind_crate() {
        let before = board("#@c#");
        assert_eq!(before.step(Dir::Right), None);
    }

    #[test]
    fn push_is_blocked_by_second_crate() {
        let before = board("#@cc.#");
        assert_eq!(before.step(Dir::Right), None);
    }

    #[test]
    fn pushing_crate_onto_last_hatch_solves_the_level() {
        let before = board("#@cH#");
        let after = before.step(Dir::Right).expect("push should be allowed");
        assert!(after.is_solved());
    }

    #[test]
    fn solved_when_every_hatch_holds_a_crate() {
        assert!(board("#@CC#").is_solved());
    }

    #[test]
    fn not_solved_while_a_hatch_is_empty() {
        assert!(!board("#@CH#").is_solved());
    }
}
