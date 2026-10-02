use std::collections::HashMap;

use super::recipes::{combine, transform};
use super::types::{Dir, Item, Mouse, Pos, StationKind, Tile};

/// The whole state of one kitchen at one moment.
#[derive(Debug, Clone, PartialEq)]
pub struct Board {
    width: i32,
    height: i32,
    /// Row by row, `width * height` long.
    tiles: Vec<Tile>,
    items: HashMap<Pos, Item>,
    chef: Pos,
    /// Always in the same order, so a mouse can be told apart by its place.
    mice: Vec<Mouse>,
    /// How many items the mice have eaten so far.
    eaten: u32,
    /// True once the chef has stepped on something hot. Nothing moves after that.
    burnt: bool,
}

/// What became of the chef's attempt to take one step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Walk {
    /// Something solid was in the way. Nothing changed.
    Blocked,
    /// The chef pushed an item off a stove without leaving their square.
    Reached,
    /// The chef is on the next square.
    Stepped,
}

impl Board {
    pub fn new(
        width: i32,
        height: i32,
        tiles: Vec<Tile>,
        items: HashMap<Pos, Item>,
        chef: Pos,
        mice: Vec<Mouse>,
    ) -> Self {
        Board {
            width,
            height,
            tiles,
            items,
            chef,
            mice,
            eaten: 0,
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

    /// Every mouse in the kitchen, always in the same order.
    pub fn mice(&self) -> &[Mouse] {
        &self.mice
    }

    /// How many items the mice have eaten since the kitchen started.
    pub fn eaten(&self) -> u32 {
        self.eaten
    }

    fn mouse_on(&self, pos: Pos) -> bool {
        self.mice.iter().any(|mouse| mouse.pos == pos)
    }

    /// True if the chef has stepped on a stove or into a flame. The kitchen
    /// is lost: the only way on is to start it again.
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
    /// A move has four phases: the chef walks (and slides on, if on grease),
    /// shoving the item in the way, then the conveyors run, and then the
    /// mice.
    pub fn step(&self, dir: Dir) -> Option<Board> {
        if self.burnt {
            return None;
        }
        let mut next = self.clone();
        match next.walk(dir) {
            Walk::Blocked => return None,
            Walk::Reached => {}
            Walk::Stepped => {
                // On grease the chef cannot stop: one step follows another
                // until they are off it, or one of them does not come off.
                while next.is_slipping() && next.walk(dir) == Walk::Stepped {}
            }
        }
        next.run_conveyors();
        next.run_mice();
        Some(next)
    }

    /// True while the chef is standing on grease, unburnt.
    fn is_slipping(&self) -> bool {
        !self.burnt && self.tile(self.chef) == Tile::Grease
    }

    /// The chef takes one step in `dir`, if nothing solid is in the way,
    /// shoving an item that is.
    fn walk(&mut self, dir: Dir) -> Walk {
        let dest = self.chef.step(dir);
        if self.tile(dest) == Tile::Wall || self.mouse_on(dest) {
            return Walk::Blocked;
        }
        let hot = matches!(
            self.tile(dest),
            Tile::Flame | Tile::Station(StationKind::Stove)
        );
        if self.item(dest).is_some() {
            // If the item in the way will not budge, neither does the chef.
            if !self.shove(dest, dir) {
                return Walk::Blocked;
            }
            // The chef reaches over a stove to push what is on it, and stays put.
            if hot {
                return Walk::Reached;
            }
        }
        self.chef = dest;
        self.burnt = hot;
        Walk::Stepped
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
        // A mouse eats whatever comes its way.
        if self.mouse_on(to) {
            self.items.remove(&from);
            self.eaten += 1;
            return true;
        }
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
            Tile::Bin | Tile::Flame => {}
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

    /// Every mouse runs one square the way it is heading, or back the way it
    /// came if it cannot, and eats the item it finds there.
    fn run_mice(&mut self) {
        for i in 0..self.mice.len() {
            let mouse = self.mice[i];
            let ways = [mouse.heading, mouse.heading.opposite()];
            let way = ways
                .into_iter()
                .find(|&way| self.mouse_can_enter(mouse.pos.step(way)));
            // With both ways shut, it waits where it is.
            if let Some(heading) = way {
                let pos = mouse.pos.step(heading);
                if self.items.remove(&pos).is_some() {
                    self.eaten += 1;
                }
                self.mice[i] = Mouse { pos, heading };
            }
        }
    }

    /// Mice keep off anything hot, and do not climb over the chef or each other.
    fn mouse_can_enter(&self, pos: Pos) -> bool {
        let shut = matches!(
            self.tile(pos),
            Tile::Wall | Tile::Flame | Tile::Station(StationKind::Stove)
        );
        !shut && pos != self.chef && !self.mouse_on(pos)
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
    use crate::puzzle::types::{Mouse, StationKind};

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
        let before = Board::new(3, 1, tiles, items, Pos::new(0, 0), Vec::new());

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
    fn walking_into_a_flame_burns_the_chef() {
        let after = pushed_right("#@!#");
        assert_eq!(after.chef(), Pos::new(2, 0));
        assert!(after.is_burnt());
    }

    #[test]
    fn an_item_pushed_into_a_flame_is_gone() {
        let after = pushed_right("#@t!#");
        assert_eq!(after.chef(), Pos::new(2, 0));
        assert_eq!(after.items().count(), 0);
        assert!(!after.is_burnt());
    }

    #[test]
    fn an_item_sliding_into_a_flame_is_gone() {
        let after = pushed_right("#@t**!.#");
        assert_eq!(after.items().count(), 0);
    }

    #[test]
    fn the_chef_slides_across_grease() {
        let after = pushed_right("#@%%%..#");
        assert_eq!(after.chef(), Pos::new(5, 0));
    }

    #[test]
    fn a_sliding_chef_stops_against_a_wall() {
        let after = pushed_right("#@%%#");
        assert_eq!(after.chef(), Pos::new(3, 0));
    }

    #[test]
    fn a_chef_resting_on_grease_can_walk_off_it() {
        let stuck = pushed_right("#@%%#\n###.#");
        let after = stuck.step(Dir::Down).expect("walking should be allowed");
        assert_eq!(after.chef(), Pos::new(3, 1));
    }

    #[test]
    fn a_sliding_chef_pushes_what_is_in_the_way() {
        let after = pushed_right("#@%%t..#");
        assert_eq!(after.chef(), Pos::new(4, 0));
        assert_eq!(after.item(Pos::new(5, 0)), Some(Item::Tomato));
    }

    #[test]
    fn a_sliding_chef_stops_behind_an_item_that_cannot_move() {
        let after = pushed_right("#@%%t#");
        assert_eq!(after.chef(), Pos::new(3, 0));
        assert_eq!(after.item(Pos::new(4, 0)), Some(Item::Tomato));
    }

    #[test]
    fn grease_carries_the_chef_onto_a_stove() {
        let after = pushed_right("#@%~.#");
        assert_eq!(after.chef(), Pos::new(3, 0));
        assert!(after.is_burnt());
    }

    #[test]
    fn a_chef_on_grease_reaching_over_a_stove_slides_no_further() {
        let before = board_with_item_at("#@%~.#", 3, Item::TomatoSoup);
        let after = before.step(Dir::Right).expect("move should be allowed");
        assert_eq!(after.chef(), Pos::new(2, 0));
        assert_eq!(after.item(Pos::new(4, 0)), Some(Item::TomatoSoup));
        assert!(!after.is_burnt());
    }

    #[test]
    fn items_do_not_slide_on_grease() {
        let after = pushed_right("#@t%%.#");
        assert_eq!(after.item(Pos::new(3, 0)), Some(Item::Tomato));
    }

    /// Where the only mouse of `board` is.
    fn mouse(board: &Board) -> Mouse {
        assert_eq!(board.mice().len(), 1);
        board.mice()[0]
    }

    #[test]
    fn a_mouse_runs_one_square_each_time_the_chef_moves() {
        let after = pushed_right("#@..#\n#-..#");
        assert_eq!(mouse(&after), Mouse { pos: Pos::new(2, 1), heading: Dir::Right });
    }

    #[test]
    fn a_mouse_does_not_run_when_the_chef_cannot_move() {
        let before = board("#.@#\n#-..#");
        assert_eq!(before.step(Dir::Right), None);
    }

    #[test]
    fn a_mouse_turns_round_at_a_wall() {
        let after = pushed_right("#@..#\n#.-#");
        assert_eq!(mouse(&after), Mouse { pos: Pos::new(1, 1), heading: Dir::Left });
    }

    #[test]
    fn an_up_and_down_mouse_runs_down_first() {
        let after = pushed_right("#@.#\n#|.#\n#..#");
        assert_eq!(mouse(&after), Mouse { pos: Pos::new(1, 2), heading: Dir::Down });
    }

    #[test]
    fn a_mouse_turns_round_at_a_stove_and_at_a_flame() {
        for text in ["#@..#\n#.-~#", "#@..#\n#.-!#"] {
            let after = pushed_right(text);
            assert_eq!(mouse(&after).pos, Pos::new(1, 1), "{text}");
        }
    }

    #[test]
    fn a_mouse_turns_round_at_the_chef() {
        let away = pushed_right("#-.@.#");
        assert_eq!(mouse(&away).pos, Pos::new(2, 0));
        // The chef steps back into the mouse's way.
        let back = away.step(Dir::Left).expect("walking should be allowed");
        assert_eq!(mouse(&back), Mouse { pos: Pos::new(1, 0), heading: Dir::Left });
    }

    #[test]
    fn a_mouse_with_nowhere_to_go_stays_put() {
        let after = pushed_right("#@..#\n#-###");
        assert_eq!(mouse(&after), Mouse { pos: Pos::new(1, 1), heading: Dir::Right });
    }

    #[test]
    fn the_chef_cannot_walk_into_a_mouse() {
        let before = board("#@-#");
        assert_eq!(before.step(Dir::Right), None);
    }

    #[test]
    fn a_mouse_stops_a_sliding_chef() {
        let after = pushed_right("#@%%|#\n####.#");
        assert_eq!(after.chef(), Pos::new(3, 0));
    }

    #[test]
    fn a_mouse_eats_the_food_it_runs_into() {
        let after = pushed_right("#@..#\n#-t.#");
        assert_eq!(mouse(&after).pos, Pos::new(2, 1));
        assert_eq!(after.items().count(), 0);
    }

    #[test]
    fn food_pushed_into_a_mouse_is_eaten() {
        let after = pushed_right("#@t-.#");
        assert_eq!(after.chef(), Pos::new(2, 0));
        assert_eq!(after.items().count(), 0);
    }

    #[test]
    fn food_carried_into_a_mouse_is_eaten() {
        let after = pushed_right("#@t>|.#\n####.##");
        assert_eq!(after.items().count(), 0);
    }

    #[test]
    fn what_the_mice_eat_is_counted() {
        assert_eq!(pushed_right("#@..#\n#-t.#").eaten(), 1);
        assert_eq!(pushed_right("#@t-.#").eaten(), 1);
    }

    #[test]
    fn an_item_lost_some_other_way_is_not_counted_as_eaten() {
        assert_eq!(pushed_right("#@tx#\n#-..#").eaten(), 0);
    }

    #[test]
    fn two_mice_do_not_share_a_square() {
        let after = pushed_right("#@...#\n#--.#");
        let squares: Vec<Pos> = after.mice().iter().map(|mouse| mouse.pos).collect();
        assert_eq!(squares, vec![Pos::new(1, 1), Pos::new(3, 1)]);
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
