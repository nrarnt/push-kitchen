use std::collections::HashMap;

use super::recipes::{combine, transform};
use super::types::{Dir, Item, Pos, StationKind, Tile};

/// The whole state of one kitchen at one moment.
#[derive(Debug, Clone, PartialEq)]
pub struct Board {
    width: i32,
    height: i32,
    /// Row by row, `width * height` long.
    tiles: Vec<Tile>,
    items: HashMap<Pos, Item>,
    chef: Pos,
    /// True once the chef has stepped on something hot. Nothing moves after that.
    burnt: bool,
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
            burnt: false,
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

    /// True if the chef has stepped on a stove. The kitchen is lost: the
    /// only way on is to start it again.
    pub fn is_burnt(&self) -> bool {
        self.burnt
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

    /// Every item in the kitchen with the square it is on, in no particular order.
    pub fn items(&self) -> impl Iterator<Item = (Pos, Item)> {
        self.items.iter().map(|(&pos, &item)| (pos, item))
    }

    /// The board after the chef tries to move one square in `dir`,
    /// or `None` if the move is not allowed.
    ///
    /// A move has three phases: the chef walks, the item in the way (if any)
    /// is shoved, and then the conveyors run.
    pub fn step(&self, dir: Dir) -> Option<Board> {
        if self.burnt {
            return None;
        }
        let dest = self.chef.step(dir);
        if self.tile(dest) == Tile::Wall {
            return None;
        }

        let mut next = self.clone();
        let stove = self.tile(dest) == Tile::Station(StationKind::Stove);
        if next.item(dest).is_some() {
            // If the item in the way will not budge, neither does the chef.
            if !next.shove(dest, dir) {
                return None;
            }
            // The chef reaches over a stove to push what is on it, and stays put.
            if !stove {
                next.chef = dest;
            }
        } else {
            next.chef = dest;
            next.burnt = stove;
        }
        next.run_conveyors();
        Some(next)
    }

    /// Moves the item on `from` one square in `dir`, if nothing is in the way.
    /// What it lands on may combine with it, cook it or swallow it.
    fn advance(&mut self, from: Pos, dir: Dir) -> bool {
        let to = from.step(dir);
        if self.tile(to) == Tile::Wall || to == self.chef {
            return false;
        }
        let Some(moving) = self.item(from) else {
            return false;
        };
        // An item already there blocks the way, unless the two combine.
        let landed = match self.item(to) {
            Some(waiting) => match combine(moving, waiting) {
                Some(combined) => combined,
                None => return false,
            },
            None => moving,
        };

        self.items.remove(&from);
        match self.tile(to) {
            Tile::Bin => {}
            Tile::Station(station) => {
                self.items.insert(to, transform(station, landed).unwrap_or(landed));
            }
            _ => {
                self.items.insert(to, landed);
            }
        }
        true
    }

    /// Pushes the item on `from` one square in `dir`, and on across any ice.
    /// Returns false if it could not move at all.
    fn shove(&mut self, from: Pos, dir: Dir) -> bool {
        let mut pos = from;
        while self.advance(pos, dir) {
            pos = pos.step(dir);
            if self.tile(pos) != Tile::Ice {
                break;
            }
        }
        pos != from
    }

    /// Lets the conveyors carry what is on them, until nothing more can move.
    fn run_conveyors(&mut self) {
        // A ring of conveyors would carry an item round forever, so the
        // number of moves is limited.
        for _ in 0..self.width * self.height {
            let mut riders: Vec<(Pos, Dir)> = self
                .items
                .keys()
                .filter_map(|&pos| match self.tile(pos) {
                    Tile::Conveyor(dir) => Some((pos, dir)),
                    _ => None,
                })
                .collect();
            // Always in the same order: a `HashMap` hands out its keys in a
            // different order each run, and two items heading for the same
            // square must not race.
            riders.sort_by_key(|(pos, _)| (pos.y, pos.x));

            let moved = riders.into_iter().any(|(pos, dir)| self.shove(pos, dir));
            if !moved {
                return;
            }
        }
    }

    /// True when every hatch holds the dish it wants. A burnt kitchen is
    /// never solved.
    pub fn is_solved(&self) -> bool {
        if self.burnt {
            return false;
        }
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
    fn items_lists_every_item_with_its_square() {
        let board = board("#@t.b#");
        let mut items: Vec<(Pos, Item)> = board.items().collect();
        items.sort_by_key(|(pos, _)| pos.x);
        assert_eq!(
            items,
            vec![(Pos::new(2, 0), Item::Tomato), (Pos::new(4, 0), Item::Bread)]
        );
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

    /// The board drawn by `text`, with `item` put on the square at `x` in
    /// the first row. For items on stations, which a drawing cannot show.
    fn board_with_item_at(text: &str, x: i32, item: Item) -> Board {
        let mut board = board(text);
        board.items.insert(Pos::new(x, 0), item);
        board
    }

    #[test]
    fn walking_onto_an_empty_stove_burns_the_chef() {
        let after = pushed_right("#@~#");
        assert_eq!(after.chef(), Pos::new(2, 0));
        assert!(after.is_burnt());
    }

    #[test]
    fn a_kitchen_does_not_start_burnt() {
        assert!(!board("#@~#").is_burnt());
    }

    #[test]
    fn a_burnt_chef_cannot_move() {
        let burnt = pushed_right("#.@~#");
        assert_eq!(burnt.step(Dir::Left), None);
    }

    #[test]
    fn pushing_an_item_off_a_stove_leaves_the_chef_where_they_are() {
        let before = board_with_item_at("#@~.#", 2, Item::TomatoSoup);
        let after = before.step(Dir::Right).expect("push should be allowed");
        assert_eq!(after.chef(), Pos::new(1, 0));
        assert_eq!(after.item(Pos::new(2, 0)), None);
        assert_eq!(after.item(Pos::new(3, 0)), Some(Item::TomatoSoup));
        assert!(!after.is_burnt());
    }

    #[test]
    fn an_item_stuck_on_a_stove_keeps_the_chef_off_it() {
        let before = board_with_item_at("#@~#", 2, Item::TomatoSoup);
        assert_eq!(before.step(Dir::Right), None);
    }

    #[test]
    fn an_item_pushed_off_a_stove_slides_over_ice() {
        let before = board_with_item_at("#@~**.#", 2, Item::TomatoSoup);
        let after = before.step(Dir::Right).expect("push should be allowed");
        assert_eq!(after.chef(), Pos::new(1, 0));
        assert_eq!(after.item(Pos::new(5, 0)), Some(Item::TomatoSoup));
    }

    #[test]
    fn a_burnt_kitchen_is_not_solved() {
        // Every hatch is served, and then the chef walks onto the stove.
        let served = pushed_right("#@tT#\n#~###");
        assert!(served.is_solved());
        let burnt = served.step(Dir::Left).and_then(|board| board.step(Dir::Down));
        assert!(!burnt.expect("walking should be allowed").is_solved());
    }

    #[test]
    fn chef_walks_over_ice_a_conveyor_and_the_bin_like_floor() {
        for text in ["#@*.#", "#@>.#", "#@<.#", "#@x.#"] {
            assert_eq!(pushed_right(text).chef(), Pos::new(2, 0), "{text}");
        }
    }

    #[test]
    fn item_pushed_onto_ice_slides_until_it_leaves_the_ice() {
        let after = pushed_right("#@t**..#");
        assert_eq!(after.chef(), Pos::new(2, 0));
        assert_eq!(after.item(Pos::new(5, 0)), Some(Item::Tomato));
    }

    #[test]
    fn sliding_item_stops_against_a_wall() {
        let after = pushed_right("#@t**#");
        assert_eq!(after.item(Pos::new(4, 0)), Some(Item::Tomato));
    }

    #[test]
    fn sliding_item_stops_against_an_item_it_has_no_recipe_with() {
        let after = pushed_right("#@t**b#");
        assert_eq!(after.item(Pos::new(4, 0)), Some(Item::Tomato));
        assert_eq!(after.item(Pos::new(5, 0)), Some(Item::Bread));
    }

    #[test]
    fn sliding_item_combines_with_its_partner() {
        let after = pushed_right("#@b**c#");
        assert_eq!(after.item(Pos::new(5, 0)), Some(Item::Sandwich));
        assert_eq!(after.items().count(), 1);
    }

    #[test]
    fn conveyor_carries_an_item_to_its_end() {
        let after = pushed_right("#@t>>..#");
        assert_eq!(after.item(Pos::new(5, 0)), Some(Item::Tomato));
    }

    #[test]
    fn conveyor_can_turn_a_corner() {
        let after = pushed_right("@t>v\n....");
        assert_eq!(after.item(Pos::new(3, 1)), Some(Item::Tomato));
    }

    #[test]
    fn conveyor_carries_an_item_onto_a_station_to_be_cooked() {
        let after = pushed_right("#@t>/#");
        assert_eq!(after.item(Pos::new(4, 0)), Some(Item::ChoppedTomato));
    }

    #[test]
    fn item_leaving_a_conveyor_onto_ice_keeps_sliding() {
        let after = pushed_right("#@t>**.#");
        assert_eq!(after.item(Pos::new(6, 0)), Some(Item::Tomato));
    }

    #[test]
    fn conveyor_cannot_carry_an_item_into_another() {
        let after = pushed_right("#@t>b#");
        assert_eq!(after.item(Pos::new(3, 0)), Some(Item::Tomato));
        assert_eq!(after.item(Pos::new(4, 0)), Some(Item::Bread));
    }

    #[test]
    fn conveyor_cannot_carry_an_item_into_the_chef() {
        // The belt points back at the square the chef has just stepped onto.
        let after = pushed_right("#@t<#");
        assert_eq!(after.chef(), Pos::new(2, 0));
        assert_eq!(after.item(Pos::new(3, 0)), Some(Item::Tomato));
    }

    #[test]
    fn item_waiting_on_a_conveyor_rides_on_once_the_way_is_clear() {
        let start = board(".....\n@t>b.\n.....");
        // The tomato goes onto the belt and waits behind the bread.
        let waiting = start.step(Dir::Right).unwrap();
        assert_eq!(waiting.item(Pos::new(2, 1)), Some(Item::Tomato));

        // Round the bottom, push the bread up, and step out of the way.
        let moves = [Dir::Down, Dir::Right, Dir::Right, Dir::Up, Dir::Right];
        let mut after = waiting;
        for dir in moves {
            after = after.step(dir).expect("move should be allowed");
        }
        assert_eq!(after.item(Pos::new(3, 1)), Some(Item::Tomato));
        assert_eq!(after.item(Pos::new(3, 0)), Some(Item::Bread));
    }

    #[test]
    fn a_ring_of_conveyors_does_not_run_forever() {
        let after = pushed_right("@t>v\n..^<");
        assert_eq!(after.items().count(), 1);
    }

    #[test]
    fn the_bin_swallows_an_item() {
        let after = pushed_right("#@tx#");
        assert_eq!(after.chef(), Pos::new(2, 0));
        assert_eq!(after.items().count(), 0);
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
