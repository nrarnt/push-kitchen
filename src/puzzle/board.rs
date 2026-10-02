use std::collections::HashMap;

use super::recipes::{combine, transform};
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
        if let Some(pushed) = next.items.remove(&dest) {
            // The chef walks into an item and shoves it one square further.
            let beyond = dest.step(dir);
            if self.tile(beyond) == Tile::Wall {
                return None;
            }
            // An item already there blocks the push, unless the two combine.
            let landed = match self.item(beyond) {
                Some(waiting) => combine(pushed, waiting)?,
                None => pushed,
            };
            // A station cooks what lands on it, if it has a use for it.
            let landed = match self.tile(beyond) {
                Tile::Station(station) => transform(station, landed).unwrap_or(landed),
                _ => landed,
            };
            next.items.insert(beyond, landed);
        }
        next.chef = dest;
        Some(next)
    }

    /// True when every hatch holds the dish it wants.
    pub fn is_solved(&self) -> bool {
        for y in 0..self.height {
            for x in 0..self.width {
                let pos = Pos::new(x, y);
                if let Tile::Hatch(dish) = self.tile(pos)
                    && self.item(pos) != Some(dish)
                {
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
    use crate::puzzle::types::StationKind;

    fn board(text: &str) -> Board {
        parse(text).expect("test level should parse")
    }

    /// The board after pushing right once from `text`.
    fn pushed_right(text: &str) -> Board {
        board(text).step(Dir::Right).expect("push should be allowed")
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
    fn chef_walks_over_a_station() {
        let after = pushed_right("#@/#");
        assert_eq!(after.chef(), Pos::new(2, 0));
    }

    #[test]
    fn chef_pushes_item_onto_empty_floor() {
        let before = board("#@t.#");
        assert_eq!(before.step(Dir::Right), Some(board("#.@t#")));
    }

    #[test]
    fn push_is_blocked_by_wall_behind_item() {
        let before = board("#@t#");
        assert_eq!(before.step(Dir::Right), None);
    }

    #[test]
    fn push_is_blocked_by_an_item_it_has_no_recipe_with() {
        let before = board("#@tb.#");
        assert_eq!(before.step(Dir::Right), None);
    }

    #[test]
    fn station_transforms_an_item_that_lands_on_it() {
        let after = pushed_right("#@t/#");
        assert_eq!(after.item(Pos::new(3, 0)), Some(Item::ChoppedTomato));
    }

    #[test]
    fn station_leaves_an_item_it_has_no_use_for_unchanged() {
        let after = pushed_right("#@b/#");
        assert_eq!(after.item(Pos::new(3, 0)), Some(Item::Bread));
    }

    #[test]
    fn item_can_be_pushed_off_a_station_again() {
        let after = pushed_right("#@t/.#").step(Dir::Right).expect("second push");
        assert_eq!(after.item(Pos::new(3, 0)), None);
        assert_eq!(after.item(Pos::new(4, 0)), Some(Item::ChoppedTomato));
    }

    #[test]
    fn pushing_an_item_into_its_partner_combines_them() {
        let after = pushed_right("#@bc#");
        assert_eq!(after.chef(), Pos::new(2, 0));
        assert_eq!(after.item(Pos::new(2, 0)), None);
        assert_eq!(after.item(Pos::new(3, 0)), Some(Item::Sandwich));
    }

    #[test]
    fn items_combined_on_a_station_are_transformed_by_it() {
        // Chef, bread, then cheese resting on a stove: not drawable as text.
        let tiles = vec![Tile::Floor, Tile::Floor, Tile::Station(StationKind::Stove)];
        let items = HashMap::from([
            (Pos::new(1, 0), Item::Bread),
            (Pos::new(2, 0), Item::Cheese),
        ]);
        let before = Board::new(3, 1, tiles, items, Pos::new(0, 0));

        let after = before.step(Dir::Right).expect("push should be allowed");
        assert_eq!(after.item(Pos::new(2, 0)), Some(Item::Toastie));
    }

    #[test]
    fn not_solved_while_a_hatch_is_empty() {
        assert!(!board("#@tT#").is_solved());
    }

    #[test]
    fn hatch_is_served_by_the_dish_it_wants() {
        assert!(pushed_right("#@tT#").is_solved());
    }

    #[test]
    fn hatch_is_not_served_by_another_dish() {
        assert!(!pushed_right("#@bT#").is_solved());
    }

    #[test]
    fn solved_only_when_every_hatch_is_served() {
        let one_served = board("#Tt@bB#").step(Dir::Left).expect("push left");
        assert!(!one_served.is_solved());

        let both_served = one_served
            .step(Dir::Right)
            .and_then(|board| board.step(Dir::Right))
            .expect("walk back and push right");
        assert!(both_served.is_solved());
    }
}
